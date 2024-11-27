# Tests basic WHERE clauses.

> CREATE TABLE test (id INT PRIMARY KEY, value STRING)
> INSERT INTO test VALUES (1, 'a'), (2, 'b'), (3, 'c')
---
ok

# Constant TRUE and FALSE filters work as expected.
[plan]> SELECT * FROM test where true
---
Filter: TRUE
└─ Scan: test
1, 'a'
2, 'b'
3, 'c'


[plan]> SELECT * FROM test where false
---
Filter: FALSE
└─ Scan: test

[plan]> SELECT * FROM test where id > 1
---
Filter: test.id > 1
└─ Scan: test
2, 'b'
3, 'c'

[plan]> SELECT * FROM test where value = 'a'
---
Filter: test.value = 'a'
└─ Scan: test
1, 'a'
