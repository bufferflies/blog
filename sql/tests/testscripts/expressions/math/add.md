# Tests the + addition operator.

# Simple integer addition.
[expr]> 1 + 2
[expr]> 1 + -3
[expr]> 1 + -2 + 3
---
3 ← Add(Constant(Integer(1)), Constant(Integer(2)))
-2 ← Add(Constant(Integer(1)), Negate(Constant(Integer(3))))
2 ← Add(Add(Constant(Integer(1)), Negate(Constant(Integer(2)))), Constant(Integer(3)))

# Simple float addition.
[expr]> 3.1 + 2.71
[expr]> 3.1 + -2.71
---
5.8100000000000005 ← Add(Constant(Float(3.1)), Constant(Float(2.71)))
0.3900000000000001 ← Add(Constant(Float(3.1)), Negate(Constant(Float(2.71))))

# Combined int/float addition yields floats.
> 3.72 + 1
> 1 + 3.72
> 1 + 3.0
> -1 + 3.72
---
4.720000000000001
4.720000000000001
4.0
2.72

# Addition with nulls yields null.
> 1 + NULL
> NULL + 3.14
> NULL + NULL
---
NULL
NULL
NULL

# Addition with infinity and NaN.
> 1 + INFINITY
> 1 + -INFINITY
> -1 + INFINITY
> 1 + NAN
> 3.14 + -NAN
> INFINITY + NAN
---
inf
-inf
inf
NaN
NaN
NaN

# Overflow and underflow.
!> 9223372036854775807 + 1
!> -9223372036854775807 + -2
> 9223372036854775807 + 1.0
> 2e308 + 2e308
---
Error: invalid input: integer overflow
Error: invalid input: integer overflow
9.223372036854776e18
inf
