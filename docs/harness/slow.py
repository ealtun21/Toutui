#!/usr/bin/env python3
"""A proxy that gives every request of the sandbox a delay, and that writes the
path and the time of each request.

**This file gives the rounds of the start with no line of code of the program.**
The measurement of T-129 read three rounds of requests in the file of this proxy:
the libraries, then the shelves of the Home view and the account, then the series,
the collections, the playlists, and the items. The start holds two rounds now.

    python3 docs/harness/slow.py 13500 13399 0.5 requests.log &

The pool of the program takes that address with a block `[[servers]]` of
`config.toml`. **The block holds for a server whose endpoints hold the stored
address**, therefore the slow address stands first and the real address after it:

    [[servers]]
    name = "sandbox"
    endpoints = [
      { url = "http://127.0.0.1:13500", priority = 0 },
      { url = "http://localhost:13399", priority = 1 },
    ]

The header of the program then says the slow address (T-105). See the traps 68 and
71 of docs/HANDOVER.md, and T-127 and T-129.
"""
import asyncio
import sys
import time

PORT = int(sys.argv[1])
TARGET = int(sys.argv[2])
DELAY = float(sys.argv[3])
LOG = open(sys.argv[4], "w", buffering=1)
START = time.monotonic()


def note(text):
    LOG.write("%8.3f %s\n" % (time.monotonic() - START, text))


async def to_the_server(reader, writer, name):
    while True:
        data = await reader.read(65536)
        if not data:
            break
        first = data.split(b"\r\n", 1)[0][:120].decode("latin1")
        if first[:3] in ("GET", "POS", "PAT", "DEL", "PUT", "HEA"):
            note("%s %s" % (name, first.rsplit(" HTTP", 1)[0]))
        await asyncio.sleep(DELAY)
        writer.write(data)
        await writer.drain()
    writer.close()


async def to_the_client(reader, writer):
    while True:
        data = await reader.read(65536)
        if not data:
            break
        writer.write(data)
        await writer.drain()
    writer.close()


COUNT = 0


async def one_connection(client_reader, client_writer):
    global COUNT
    COUNT += 1
    name = "c%d" % COUNT
    note("%s the connection opened" % name)
    try:
        server_reader, server_writer = await asyncio.open_connection("127.0.0.1", TARGET)
    except OSError as error:
        note("%s no connection to the server: %s" % (name, error))
        client_writer.close()
        return
    await asyncio.gather(
        to_the_server(client_reader, server_writer, name),
        to_the_client(server_reader, client_writer),
        return_exceptions=True,
    )


async def main():
    server = await asyncio.start_server(one_connection, "127.0.0.1", PORT)
    note("the proxy holds the port %d, and every request waits %.3f s" % (PORT, DELAY))
    async with server:
        await server.serve_forever()


asyncio.run(main())
