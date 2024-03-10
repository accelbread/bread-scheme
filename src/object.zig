// bread-scheme -- R7RS Scheme interpreter
// Copyright (C) 2023 Archit Gupta <archit@accelbread.com>
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU Affero General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version.
//
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more
// details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.
//
// SPDX-License-Identifier: AGPL-3.0-or-later

const std = @import("std");
const Allocator = std.mem.Allocator;

const Pair = struct {
    car: *Object,
    cdr: *Object,

    pub fn format(
        self: Pair,
        comptime _: []const u8,
        _: std.fmt.FormatOptions,
        writer: anytype,
    ) !void {
        try writer.print("({}", .{self.car.*});
        var next = self.cdr.*;
        while (next == .pair) {
            try writer.print(" {}", .{next.pair.car.*});
            next = next.pair.cdr.*;
        }
        if (next != .empty) {
            try writer.print(" . {}", .{next});
        }
        try writer.print(")", .{});
    }
};

pub const Object = union(enum) {
    boolean: bool,
    char: u8,
    empty: void,
    eof: void,
    number: i64,
    pair: Pair,
    string: []const u8,
    symbol: []const u8,

    pub fn format(
        self: Object,
        comptime _: []const u8,
        _: std.fmt.FormatOptions,
        writer: anytype,
    ) !void {
        try switch (self) {
            .boolean => |val| writer.print("{s}", .{if (val) "#t" else "#f"}),
            .char => |val| writer.print("#\\{X}", .{val}),
            .empty => writer.print("()", .{}),
            .pair => |val| writer.print("{}", .{val}),
            .symbol => |val| writer.print("{s}", .{val}),
            .number => |val| writer.print("{}", .{val}),
            .string => |val| writer.print("\"{s}\"", .{val}),
            .eof => writer.print("#<eof>", .{}),
        };
    }

    pub fn factory(gpa: Allocator) Factory {
        return .{ .gpa = gpa };
    }

    const Factory = struct {
        gpa: Allocator,

        pub fn boolean(self: Factory, value: bool) !*Object {
            var ret = try self.gpa.create(Object);
            ret.* = .{ .boolean = value };
            return ret;
        }

        pub fn char(self: Factory, value: u8) !*Object {
            var ret = try self.gpa.create(Object);
            ret.* = .{ .char = value };
            return ret;
        }

        pub fn empty(self: Factory) !*Object {
            var ret = try self.gpa.create(Object);
            ret.* = .empty;
            return ret;
        }

        pub fn cons(self: Factory, car: *Object, cdr: *Object) !*Object {
            var ret = try self.gpa.create(Object);
            ret.* = .{ .pair = .{ .car = car, .cdr = cdr } };
            return ret;
        }

        pub fn symbol(self: Factory, value: []const u8) !*Object {
            var ret = try self.gpa.create(Object);
            ret.* = .{ .symbol = value };
            return ret;
        }

        pub fn number(self: Factory, value: i64) !*Object {
            var ret = try self.gpa.create(Object);
            ret.* = .{ .number = value };
            return ret;
        }

        pub fn string(self: Factory, value: []const u8) !*Object {
            var ret = try self.gpa.create(Object);
            ret.* = .{ .string = value };
            return ret;
        }

        pub fn eof(self: Factory) !*Object {
            var ret = try self.gpa.create(Object);
            ret.* = .eof;
            return ret;
        }
    };
};

test "allocate objects" {
    var alloc = Object.factory(std.testing.allocator);
    var v1 = try alloc.empty();
    defer std.testing.allocator.destroy(v1);
    var v2 = try alloc.string("hi");
    defer std.testing.allocator.destroy(v2);
    var v3 = try alloc.cons(v1, v2);
    defer std.testing.allocator.destroy(v3);
}
