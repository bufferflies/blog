> CREATE TABLE test (id INT PRIMARY KEY, value STRING)
> INSERT INTO test VALUES (1, 'a'), (2, 'b'), (3, 'c')
---
ok


[opt]> SELECT * FROM test WHERE test.id=1 AND test.value = 'a'
---
Initial:
   Filter: test.id = 1 AND test.value = 'a'
   └─ Scan: test
Filter pushdown:
   Scan: test (test.id = 1 AND test.value = 'a')
1, 'a'
