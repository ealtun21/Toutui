#!/usr/bin/env python3
"""A proxy that takes one field away from one row of a list of the answer.

`a_field_of_the_answer_goes_away.py` takes a field out at every depth of the
body (T-177): that proxy gives a server of another version, which holds one
field fewer in every row. **This proxy gives one row of less information**, and
every other row of the same list keeps every field. A measurement then sees what
the program does with the rows that it kept.

    python3 docs/harness/a_field_of_one_row_goes_away.py 13506 13399 \\
        requests.log /api/items/<the id> media.audioFiles 1 ino

The arguments: the port of this proxy, the port of the sandbox, the file of the
log, the path, the dotted name of the list, the number of the row of that list
(0 is the first row), and one name of a field or more. Every other request goes
to the sandbox, therefore the login works through this proxy and the token is a
real token.

**A name of numbers alone is the row of a list** (T-192): the name
`results.0.books` is the list of the books of the first collection of the
answer.

**The name `.` says that the body itself is the list** (T-190). The answer of
`GET /api/libraries/:id/personalized` is a bare array of the shelves, and no
field of an object holds it:

    python3 docs/harness/a_field_of_one_row_goes_away.py 13506 13399 \\
        requests.log /api/libraries/<the id>/personalized . 1 label

**Give the absolute path of the file of the log** (the trap 132), and give the
account one address alone (the trap 129).
"""
import asyncio
import json
import sys
import time

PORT = int(sys.argv[1])
TARGET = int(sys.argv[2])
LOG = open(sys.argv[3], "w", buffering=1)
THE_PATH = sys.argv[4]
# The name `.` says that the body itself is the list.
THE_LIST = [] if sys.argv[5] == "." else sys.argv[5].split(".")
THE_ROW = int(sys.argv[6])
THE_FIELDS = set(sys.argv[7:])
# The name of the list for the log.
THE_NAME = sys.argv[5]
START = time.monotonic()


def note(text):
    LOG.write("%8.3f %s\n" % (time.monotonic() - START, text))


def without_the_fields(value):
    """Takes the fields out of the one row of the list. It gives the body."""
    row = value

    for name in THE_LIST:
        # A name of numbers alone is the row of a list. `results.0.books` is
        # therefore the list of the books of the first collection (T-192).
        if name.isdigit() and isinstance(row, list) and len(row) > int(name):
            row = row[int(name)]
            continue

        if not isinstance(row, dict) or name not in row:
            note("!! the list %s is not in the body" % THE_NAME)
            return value
        row = row[name]

    if not isinstance(row, list) or len(row) <= THE_ROW:
        note("!! the list %s holds no row %d" % (THE_NAME, THE_ROW))
        return value

    row = row[THE_ROW]

    if isinstance(row, dict):
        for field in THE_FIELDS:
            row.pop(field, None)

    return value


async def one_request(reader):
    """Reads one request. It gives the head, the body, and the path."""
    head = await reader.readuntil(b"\r\n\r\n")
    lines = head.split(b"\r\n")
    first = lines[0].split(b" ")
    path = first[1].decode("latin1") if len(first) > 1 else ""

    length = 0
    for line in lines[1:]:
        if line.lower().startswith(b"content-length:"):
            length = int(line.split(b":", 1)[1].strip())

    body = await reader.readexactly(length) if length else b""
    return head, body, path


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


def the_body_of_the_answer(answer):
    """Gives the head and the body of an answer. The body of a chunked answer
    comes together here, because the answer of this proxy holds a length."""
    head, _, body = answer.partition(b"\r\n\r\n")
    if b"chunked" not in head.lower():
        return head, body

    whole = b""
    while True:
        line, _, body = body.partition(b"\r\n")
        size = int(line.split(b";")[0], 16)
        if size == 0:
            break
        whole += body[:size]
        body = body[size + 2 :]
    return head, whole


def the_answer_again(head, body):
    """Makes an answer of a head and of a body, with the length of that body."""
    lines = [
        line
        for line in head.split(b"\r\n")
        if not line.lower().startswith(b"content-length:")
        and not line.lower().startswith(b"transfer-encoding:")
        and not line.lower().startswith(b"connection:")
    ]
    lines.append(b"Content-Length: %d" % len(body))
    lines.append(b"Connection: close")
    return b"\r\n".join(lines) + b"\r\n\r\n" + body


async def one_connection(client_reader, client_writer):
    try:
        while True:
            head, body, path = await one_request(client_reader)

            answer = await to_the_sandbox(head, body)

            if path.split("?")[0].rstrip("/") == THE_PATH.rstrip("/"):
                the_head, the_body = the_body_of_the_answer(answer)
                try:
                    value = json.loads(the_body)
                except ValueError:
                    note("!! %s: the answer holds no JSON" % path)
                else:
                    the_body = json.dumps(without_the_fields(value)).encode()
                    answer = the_answer_again(the_head, the_body)
                    note(
                        "<< %s: %s goes away of the row %d of %s"
                        % (
                            path,
                            ", ".join(sorted(THE_FIELDS)),
                            THE_ROW,
                            THE_NAME,
                        )
                    )
            else:
                note("-- %s" % path)

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
        "the proxy holds the port %d, and %s goes away of the row %d of %s"
        % (PORT, ", ".join(sorted(THE_FIELDS)), THE_ROW, THE_NAME)
    )
    async with server:
        await server.serve_forever()


asyncio.run(main())
