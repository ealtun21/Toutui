#!/usr/bin/env python3
"""A proxy that gives one fault to one method of a path, and forwards the rest.

`one_path_fails.py` fails **every** request of a path. A key of the program that
reads a state of the server and that then writes it uses **one** path with two
methods: `GET /api/me/progress/:id` and `PATCH /api/me/progress/:id`. A proxy
that fails both of them says nothing of the read, because the write says the
fault. This proxy fails the read alone, therefore the program writes with a
state that it did not read.

    python3 docs/harness/one_method_fails.py 13500 13399 requests.log \
        GET:/api/me/progress

The arguments: the port of this proxy, the port of the sandbox, the file of the
log, and then one or more rules. A rule is `METHOD:part-of-a-path`, or a
`part-of-a-path` alone for every method. A request that agrees with one rule
takes the answer `500`, and every other request goes to the sandbox.

The address of the account must hold this proxy alone (the trap 129), and the
answer `500` keeps the state `Up` (T-128). The other traps of
`one_path_fails.py` are the traps of this file too: give the absolute path of
the log, because a `cd` of the same command line takes this program with it.
"""
import asyncio
import sys
import time

PORT = int(sys.argv[1])
TARGET = int(sys.argv[2])
LOG = open(sys.argv[3], "w", buffering=1)
START = time.monotonic()

RULES = []
for rule in sys.argv[4:]:
    if ":" in rule and not rule.startswith("/"):
        method, part = rule.split(":", 1)
        RULES.append((method.upper(), part))
    else:
        RULES.append((None, rule))

THE_FAULT = (
    b"HTTP/1.1 500 Internal Server Error\r\n"
    b"Content-Type: text/plain\r\n"
    b"Content-Length: 21\r\n"
    b"Connection: keep-alive\r\n"
    b"\r\n"
    b"this proxy said no.\r\n"
)


def note(text):
    LOG.write("%8.3f %s\n" % (time.monotonic() - START, text))


def the_rule_agrees(method, path):
    return any(
        (rule_method is None or rule_method == method) and part in path
        for rule_method, part in RULES
    )


async def one_request(reader):
    """Reads one request. It gives the head, the body, the method, and the path."""
    head = await reader.readuntil(b"\r\n\r\n")
    lines = head.split(b"\r\n")
    words = lines[0].split(b" ")
    method = words[0].decode("latin1") if words else ""
    path = words[1].decode("latin1") if len(words) > 1 else ""

    length = 0
    for line in lines[1:]:
        if line.lower().startswith(b"content-length:"):
            length = int(line.split(b":", 1)[1].strip())

    body = await reader.readexactly(length) if length else b""
    return head, body, method, path


async def to_the_sandbox(head, body):
    """Sends one request to the sandbox, and it gives the whole answer."""
    reader, writer = await asyncio.open_connection("127.0.0.1", TARGET)
    head = head.replace(b"Connection: keep-alive\r\n", b"")
    head = head[:-2] + b"Connection: close\r\n\r\n"
    writer.write(head + body)
    await writer.drain()
    answer = await reader.read()
    writer.close()
    return answer


async def one_connection(client_reader, client_writer):
    try:
        while True:
            head, body, method, path = await one_request(client_reader)

            if the_rule_agrees(method, path):
                note("500 %s %s" % (method, path))
                client_writer.write(THE_FAULT)
                await client_writer.drain()
                continue

            note("--- %s %s" % (method, path))
            answer = await to_the_sandbox(head, body)
            # The answer of the sandbox says `Connection: close`, therefore this
            # connection ends with it.
            client_writer.write(answer)
            await client_writer.drain()
            break
    except (asyncio.IncompleteReadError, ConnectionResetError, BrokenPipeError):
        pass
    finally:
        client_writer.close()


async def main():
    server = await asyncio.start_server(one_connection, "127.0.0.1", PORT)
    note(
        "the proxy holds the port %d, and it fails %s"
        % (PORT, " ".join("%s %s" % (m or "every method", p) for m, p in RULES))
    )
    async with server:
        await server.serve_forever()


asyncio.run(main())
