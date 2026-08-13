#!/usr/bin/env python3
"""A port that takes a connection and that answers nothing at all.

This is the address of a house that a user is away from: the packets go, and no
machine answers them.

    python3 docs/harness/blackhole.py 13500

**A port that no program holds refuses a connection at once**, therefore it
measures no limit of time at all (the trap 112). The measurement of T-149 gave
the program this port and it read the second of the message of the fault: the
program said nothing at all before that item, and it says
`Download failed for "…": the request failed` at the second 15 now.
"""
import asyncio
import sys

PORT = int(sys.argv[1])


async def one_connection(reader, writer):
    while True:
        await asyncio.sleep(3600)


async def main():
    server = await asyncio.start_server(one_connection, "127.0.0.1", PORT)
    async with server:
        await server.serve_forever()


asyncio.run(main())
