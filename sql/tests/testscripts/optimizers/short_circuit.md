# Tests the short circuiting optimizer.

> CREATE TABLE test (id INT PRIMARY KEY, value STRING)
> INSERT INTO test VALUES (1, 'a'), (2, 'b'), (3, 'c')
---
ok

# TRUE predicates are removed.
[opt]> SELECT * FROM test WHERE TRUE
---
Initial:
   Filter: TRUE
   └─ Scan: test
Filter pushdown:
   Scan: test (TRUE)
Short circuit:
   Scan: test
1, 'a'
2, 'b'
3, 'c'

[opt]> SELECT * FROM test WHERE false
---
Initial:
   Filter: FALSE
   └─ Scan: test
Filter pushdown:
   Scan: test (FALSE)
Short circuit:
   Nothing

[opt]> SELECT * FROM test limit 0
---
Initial:
   Limit: 0
   └─ Scan: test
Short circuit:
   Nothing

[opt]> SELECT * FROM test where NULL
---
Initial:
   Filter: NULL
   └─ Scan: test
Filter pushdown:
   Scan: test (NULL)
Short circuit:
   Nothing
