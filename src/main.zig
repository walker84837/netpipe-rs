const std = @import("std");
const net = std.net;
const os = std.os;
const fs = std.fs;
const io = std.io;
const mem = std.mem;
const process = std.process;
const log = std.log;
const args = @import("args");

pub const Protocol = enum {
    tcp,
    udp,
};

pub const IpVersion = enum {
    v4,
    v6,
};

pub const Args = struct {
    file: ?[]const u8 = null,
    ip_version: IpVersion = .v4,
    protocol: Protocol = .tcp,
    timeout: u64 = 0,
    listen: bool = false,
    exec: ?[]const u8 = null,
    verbose: bool = false,
    address: ?[]const u8 = null,
    port: ?u16 = null,
};

pub const meta = .{
    .name = "netcat",
    .full_text = "A Zig port of netcat",
    .usage_summary = "[options] [address] [port]",
    .option_docs = .{
        .file = "Specify a file to use",
        .ip_version = "IP version to use (4 or 6)",
        .protocol = "Protocol to use (tcp or udp)",
        .timeout = "Timeout in seconds",
        .listen = "Listen mode",
        .exec = "Execute command",
        .verbose = "Logs to stdout",
        .address = "Address to connect/listen",
        .port = "Port to connect/listen",
    },
    .shorthands = .{
        .f = "file",
        .I = "ip_version",
        .p = "protocol",
        .t = "timeout",
        .l = "listen",
        .e = "exec",
        .v = "verbose",
    },
};

pub fn executeCommand(
    allocator: mem.Allocator,
    input: anytype,
    command: []const u8,
) !void {
    log.info("Executing command: {s}", .{command});

    var child = std.ChildProcess.init(&.{ "sh", "-c", command }, allocator);
    child.stdin_behavior = .Pipe;
    child.stdout_behavior = .Pipe;
    child.stderr_behavior = .Pipe;

    try child.spawn();

    if (child.stdin) |*stdin| {
        var buffer = std.ArrayList(u8).init(allocator);
        defer buffer.deinit();

        try input.reader().readAllArrayList(&buffer, std.math.maxInt(usize));
        try stdin.writeAll(buffer.items);
        stdin.close();
    }

    const term = try child.wait();
    const stdout = try child.stdout.?.reader().readAllAlloc(allocator, std.math.maxInt(usize));
    defer allocator.free(stdout);
    const stderr = try child.stderr.?.reader().readAllAlloc(allocator, std.math.maxInt(usize));
    defer allocator.free(stderr);

    try io.getStdOut().writeAll(stdout);
    try io.getStdErr().writeAll(stderr);

    switch (term) {
        .Exited => |code| if (code != 0) return error.CommandFailed,
        else => return error.CommandFailed,
    }
}

pub fn isValidAddress(address: []const u8, version: u8) bool {
    return switch (version) {
        4 => blk: {
            var addr: std.net.Ip4Address = undefined;
            break :blk std.net.Ip4Address.parse(address, &addr) catch false;
        },
        6 => blk: {
            var addr: std.net.Ip6Address = undefined;
            break :blk std.net.Ip6Address.parse(address, &addr) catch false;
        },
        else => false,
    };
}

fn handleTcpConnection(
    allocator: mem.Allocator,
    stream: net.Stream,
    config: *const Config,
    timeout: u64,
) !void {
    try stream.setReadTimeout(timeout);

    if (config.exec) |command| {
        try executeCommand(allocator, stream, command);
    } else if (config.file) |file_path| {
        var file = try fs.cwd().createFile(file_path, .{});
        defer file.close();

        var buffer: [8192]u8 = undefined;
        while (true) {
            const bytes_read = try stream.read(&buffer);
            if (bytes_read == 0) break;
            try file.writeAll(buffer[0..bytes_read]);
        }
    } else {
        var buffer: [8192]u8 = undefined;
        while (true) {
            const bytes_read = try stream.read(&buffer);
            if (bytes_read == 0) break;
            try io.getStdOut().writeAll(buffer[0..bytes_read]);
        }
    }
}

fn runTcpServer(
    allocator: mem.Allocator,
    config: *const Config,
    destination: []const u8,
    timeout: u64,
) !void {
    const address = try net.Address.parseIp(destination, config.port.?);
    var server = net.StreamServer.init(.{});
    try server.listen(address);
    defer server.deinit();

    log.info("Listening on {s}...", .{destination});

    while (true) {
        const connection = try server.accept();
        handleTcpConnection(allocator, connection.stream, config, timeout) catch |err| {
            log.err("Failed to handle connection: {}", .{err});
        };
    }
}

fn handleUdpConnection(
    allocator: mem.Allocator,
    socket: os.socket_t,
    config: *const Config,
    timeout: u64,
) !void {
    var buffer: [65535]u8 = undefined;
    var addr: os.sockaddr = undefined;
    var addrlen: os.socklen_t = @sizeOf(@TypeOf(addr));

    try os.setsockopt(socket, os.SOL.SOCKET, os.SO.RCVTIMEO, &std.mem.toBytes(timeout));
    const bytes_received = try os.recvfrom(socket, &buffer, 0, &addr, &addrlen);

    if (config.exec) |command| {
        var fbs = io.fixedBufferStream(buffer[0..bytes_received]);
        try executeCommand(allocator, fbs.reader(), command);
    } else if (config.file) |file_path| {
        var file = try fs.cwd().createFile(file_path, .{});
        defer file.close();
        try file.writeAll(buffer[0..bytes_received]);
    } else {
        try io.getStdOut().writeAll(buffer[0..bytes_received]);
    }
}

fn runUdpServer(
    allocator: mem.Allocator,
    config: *const Config,
    destination: []const u8,
    timeout: u64,
) !void {
    const address = try net.Address.parseIp(destination, config.port.?);
    const socket = try os.socket(address.any.family, os.SOCK.DGRAM, 0);
    defer os.closeSocket(socket);

    var sockaddr = address.any;
    try os.bind(socket, &sockaddr, @sizeOf(@TypeOf(sockaddr)));

    log.info("Listening on {s}...", .{destination});
    try handleUdpConnection(allocator, socket, config, timeout);
}

pub fn runServer(
    allocator: mem.Allocator,
    config: *const Config,
    protocol: []const u8,
    timeout: u64,
) !void {
    const destination = try std.fmt.allocPrint(allocator, "{s}:{}", .{ config.address.?, config.port.? });
    defer allocator.free(destination);

    if (mem.eql(u8, protocol, "tcp")) {
        try runTcpServer(allocator, config, destination, timeout);
    } else if (mem.eql(u8, protocol, "udp")) {
        try runUdpServer(allocator, config, destination, timeout);
    } else {
        return error.InvalidProtocol;
    }
}

fn prepareBufferFromFileOrStdin(allocator: mem.Allocator, config: *const Config) ![]u8 {
    var buffer = std.ArrayList(u8).init(allocator);
    defer buffer.deinit();

    if (config.file) |file_path| {
        var file = try fs.cwd().openFile(file_path, .{});
        defer file.close();
        try file.reader().readAllArrayList(&buffer, std.math.maxInt(usize));
    } else {
        try io.getStdIn().reader().readAllArrayList(&buffer, std.math.maxInt(usize));
    }
    return buffer.toOwnedSlice();
}

fn runTcpClient(destination: []const u8, buffer: []const u8, timeout: u64, allocator: mem.Allocator) !void {
    const stream = try net.tcpConnectToHost(allocator, destination, timeout);
    defer stream.close();
    try stream.writeAll(buffer);
}

fn runUdpClient(
    destination: []const u8,
    buffer: []const u8,
    timeout: u64,
) !void {
    const address = try net.Address.parseIp(destination, 0);
    const socket = try os.socket(address.any.family, os.SOCK.DGRAM, 0);
    defer os.closeSocket(socket);

    try os.setsockopt(socket, os.SOL.SOCKET, os.SO.SNDTIMEO, &std.mem.toBytes(timeout));

    var sockaddr = address.any;
    _ = try os.sendto(socket, buffer, 0, &sockaddr, @sizeOf(@TypeOf(sockaddr)));
}

pub fn runClient(
    allocator: mem.Allocator,
    config: *const Config,
    protocol: []const u8,
    timeout: u64,
) !void {
    const destination = try std.fmt.allocPrint(allocator, "{s}:{}", .{ config.address.?, config.port.? });
    defer allocator.free(destination);

    const buffer = try prepareBufferFromFileOrStdin(allocator, config);
    defer allocator.free(buffer);

    if (mem.eql(u8, protocol, "tcp")) {
        try runTcpClient(destination, buffer, timeout);
    } else if (mem.eql(u8, protocol, "udp")) {
        try runUdpClient(destination, buffer, timeout, allocator);
    } else {
        return error.InvalidProtocol;
    }
}

// src/main.zig
const Config = struct {
    args: Args,
    ip_num: u8,
    protocol_str: []const u8,
    timeout_duration: u64,
};

pub fn main() !void {
    var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
    defer arena.deinit();
    const allocator = arena.allocator();

    const parsed_args = try args.parseForCurrentProcess(Args, allocator, .print);
    defer parsed_args.deinit();

    const options = parsed_args.options;

    // Initialize logging
    if (options.verbose) {
        log.info("Starting application with arguments: {any}", .{options});
    }

    // Validate address and port
    if (options.listen) {
        if (options.address == null or options.port == null) {
            log.err("Listening mode requires both address and port to be specified.", .{});
            return error.MissingArguments;
        }
    } else {
        if (options.address == null or options.port == null) {
            log.err("Client mode requires both address and port to be specified.", .{});
            return error.MissingArguments;
        }
    }

    // Validate IP address
    if (options.address) |address| {
        const ip_num: u8 = switch (options.ip_version) {
            .v4 => 4,
            .v6 => 6,
        };
        if (!isValidAddress(address, ip_num)) {
            log.err("Invalid IP address: {s} for version {}", .{ address, ip_num });
            return error.InvalidAddress;
        }
    }

    const protocol_str = switch (options.protocol) {
        .tcp => "tcp",
        .udp => "udp",
    };

    const timeout_duration = options.timeout * std.time.ns_per_s;
    const config = Config{
        .args = options,
        .ip_num = switch (options.ip_version) {
            .v4 => 4,
            .v6 => 6,
        },
        .protocol_str = protocol_str,
        .timeout_duration = timeout_duration,
    };

    if (options.listen) {
        try runServer(allocator, &config.args, protocol_str, timeout_duration);
    } else {
        try runClient(allocator, &config.args, protocol_str, timeout_duration);
    }
}
