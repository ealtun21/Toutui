#!/usr/bin/env python3
"""A proxy that sends the body of the answer slowly.

    python3 docs/harness/slow_body.py 13500 13399 0.004

The delay stands between two blocks of 64 kilobytes of the answer, therefore a
download of 115 megabytes takes about seven seconds and every other request
stays fast. `docs/harness/slow.py` gives a delay of the **request**, and it
therefore leaves a download of the loopback at less than one second (the trap
111).

**The program takes this address in the block `[[servers]]` of `config.toml`**,
with the slow address first and the real address after it. The key `D` takes the
address of the pool since T-149, therefore this proxy holds the download too.

A next session needs this tool for the condition that T-148 could not measure:
the key `X` of one window while the other window downloads.
"""
import asyncio
import sys

PORT = int(sys.argv[1])
TARGET = int(sys.argv[2])
DELAY = float(sys.argv[3])


async def to_the_server(reader, writer):
    while True:
        data = await reader.read(65536)
        if not data:
            break
        writer.write(data)
        await writer.drain()
    writer.close()


async def to_the_client(reader, writer):
    blocks = 0
    while True:
        data = await reader.read(65536)
        if not data:
            break
        blocks += 1
        # The head of the answer goes at once. The body goes slowly.
        if blocks > 1:
            await asyncio.sleep(DELAY)
        writer.write(data)
        await writer.drain()
    writer.close()


async def one_connection(client_reader, client_writer):
    try:
        server_reader, server_writer = await asyncio.open_connection("127.0.0.1", TARGET)
    except OSError:
        client_writer.close()
        return
    await asyncio.gather(
        to_the_server(client_reader, server_writer),
        to_the_client(server_reader, client_writer),
        return_exceptions=True,
    )


async def main():
    server = await asyncio.start_server(one_connection, "127.0.0.1", PORT)
    async with server:
        await server.serve_forever()


asyncio.run(main())
