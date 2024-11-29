# Tests the constant folding optimizer.

> CREATE TABLE test (id INT PRIMARY KEY, value STRING)
> INSERT INTO test VALUES (1, 'a'), (2, 'b'), (3, 'c')
---
ok

# Constant folding is applied in all places where expressions are used.
[opt]> SELECT * FROM test LIMIT 1+1
---
Initial:
   Limit: 2
   └─ Scan: test
1, 'a'
2, 'b'


[opt]> SELECT * FROM test WHERE 1+1 > 1
---
Initial:
   Filter: 1 + 1 > 1
   └─ Scan: test
Constant folding:
   Filter: TRUE
   └─ Scan: test
Filter pushdown:
   Scan: test (TRUE)
Short circuit:
   Scan: test
1, 'a'
2, 'b'
3, 'c'

[opt]> SELECT * FROM test WHERE 1+1 < 1 AND id > 1
---
Initial:
   Filter: 1 + 1 < 1 AND test.id > 1
   └─ Scan: test
Constant folding:
   Filter: FALSE
   └─ Scan: test
Filter pushdown:
   Scan: test (FALSE)
Short circuit:
   Nothing

[opt]> SELECT id as name FROM test 
---
Initial:
   Projection: test.id as name
   └─ Scan: test
1
2
3
