# Tests the ^ exponentiation operator.

# Integers.
[expr]> 2 ^ 3
[expr]> 2 ^ 0
[expr]> 0 ^ 2
[expr]> 9 ^ -3
---
8 ← Exponential(Constant(Integer(2)), Constant(Integer(3)))
1 ← Exponential(Constant(Integer(2)), Constant(Integer(0)))
0 ← Exponential(Constant(Integer(0)), Constant(Integer(2)))
0.0013717421124828531 ← Exponential(Constant(Integer(9)), Negate(Constant(Integer(3))))

# Floats.
[expr]> 6.25 ^ 0.5
[expr]> 6.25 ^ 3.14
---
2.5 ← Exponential(Constant(Float(6.25)), Constant(Float(0.5)))
315.5464179407336 ← Exponential(Constant(Float(6.25)), Constant(Float(3.14)))

# Mixed.
> 6.25 ^ 2
> 9 ^ 0.5
---
39.0625
3.0

# Overflow and underflow.
!> 2 ^ 10000000000
!> 9223372036854775807 ^ 2
> 10e200 ^ 2
---
Error: invalid input: integer overflow
Error: invalid input: integer overflow
inf

# Nulls.
> 1 ^ NULL
> 3.14 ^ NULL
> NULL ^ 2
> NULL ^ 3.14
> NULL ^ NULL
---
NULL
NULL
NULL
NULL
NULL

# Infinity and NaN.
> 2 ^ INFINITY
> INFINITY ^ 2
> INFINITY ^ INFINITY
> 2 ^ -INFINITY
> 2 ^ NAN
> NAN ^ 2
> NAN ^ NAN
---
inf
inf
inf
0.0
NaN
NaN
NaN

# Bools and strings.
!> TRUE ^ FALSE
!> 'a' ^ 'b'
---
Error: invalid input: can't Exponential TRUE and FALSE
Error: invalid input: can't Exponential 'a' and 'b'
