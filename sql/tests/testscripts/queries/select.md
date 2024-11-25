# Tests the SELECT part of queries.

# Create a basic test table, and a secondary table for join column lookups.
> CREATE TABLE test ( \
    id INT PRIMARY KEY, \
    "bool" BOOLEAN, \
    "float" FLOAT, \
    "int" INT, \
    "string" STRING \
)
> INSERT INTO test VALUES (1, true, 3.14, 7, 'foo')
> INSERT INTO test VALUES (2, false, 2.718, 1, '👍')
> INSERT INTO test VALUES (3, NULL, NULL, NULL, NULL)

> CREATE TABLE other (id INT PRIMARY KEY, value STRING)
> INSERT INTO other VALUES (1, 'a'), (2, 'b')
---
ok

# Select constant values.
[plan]> select 1
---
Projection: 1
└─ Values: blank row
1

[plan]> SELECT NULL, NOT FALSE, 2^2+1, 3.14*2, 'Hi 👋'
---
Projection: NULL, TRUE, 5, 6.28, 'Hi 👋'
└─ Values: blank row
NULL, TRUE, 5, 6.28, 'Hi 👋'