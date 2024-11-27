# Tests basic WHERE clauses.

> CREATE TABLE test (id INT PRIMARY KEY, value STRING)
> INSERT INTO test VALUES (1, 'a'), (2, 'b'), (3, 'c')
---
ok

# Constant TRUE and FALSE filters work as expected.
> SELECT * FROM test
---
Scan: test
1, 'a'
2, 'b'
3, 'c'