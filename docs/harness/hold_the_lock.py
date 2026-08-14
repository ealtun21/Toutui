#!/usr/bin/env python3
"""A second writer of the database of the program. See T-199.

**A fault of the database needs no proxy and no change of the source.** This
script takes the write lock of the database of the program with
`BEGIN EXCLUSIVE`, and it holds it. rusqlite holds a busy timeout of five
seconds by default, therefore every call of the program that meets a lock of
more than five seconds comes back with `database is locked`.

That is the condition of T-140: two programs of one account write one database.

    python3 docs/harness/hold_the_lock.py \\
        $XDG_CONFIG_HOME/toutui/db.sqlite3 70

The first argument is the file of the database, and the second one is the number
of seconds of the lock. The script writes `the lock stands` when the lock is up,
and `the lock goes away` at the end of it: **a measurement must wait for the
first line**, because `BEGIN EXCLUSIVE` needs the file first.

**A lock of this script blocks a read too**, because the database of the program
holds the journal of a rollback and not a journal of write ahead. A measurement
of a read that failed and a measurement of a write that failed therefore take one
harness.

**The sequence of the two decides the condition** (the trap 94). The login of the
program holds the request of the server first, and that request takes about one
second against the sandbox: a lock that comes after the key of the password
reaches the read of `Database::new` and not the write of the row. A lock that
stands before that key reaches the write, and every key of the login screen then
takes five seconds, because the screen reads the row of its message at each key.
"""

import sqlite3
import sys
import time


def main() -> int:
    if len(sys.argv) != 3:
        print(__doc__)
        return 2

    path, seconds = sys.argv[1], float(sys.argv[2])

    connection = sqlite3.connect(path, isolation_level=None, timeout=0)
    connection.execute("BEGIN EXCLUSIVE")
    print("the lock stands", flush=True)

    try:
        time.sleep(seconds)
    finally:
        connection.execute("ROLLBACK")
        print("the lock goes away", flush=True)

    return 0


if __name__ == "__main__":
    sys.exit(main())
