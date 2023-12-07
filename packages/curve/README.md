# CURVE

## Main functionality
```The purpose of this contract is to define different type of mathematical curves orientated for blockchain applications. Those curves can be used for rewards distributions, vesting schedules and any other functions that change with time.```

## Main methods

### 1. saturating_linear
 **Parameters**:
  * min_x: `u64` value representing starting time of the curve.
  * min_y: `u128` value at the starting time.
  * max_x: `u64` value representing time when the curve saturates.
  * max_y: `u128` value at the saturation time.

**Return Type**: 
Curve
**Description:**
Ctor for Saturated curve.

### 2. constant
**Parameter**:
* y: `u128` value representing the constant value of the curve.

**Return Type**: 
Curve
**Description**: 
Ctor for constant curve.

### 3. value
**Parameter**:
 * x: `u64` value representing the point at which to evaluate the curve.
 
- **Return Type**: 
u128
- **Description**: 
provides y = f(x) evaluation.

### 4. size
 **Parameters**: 
 None.
 **Return Type**: 
 u32
 **Description**: 
 Returns the number of steps in the curve.

### 5. validate
**Parameters**:
None.
**Return Type**: 
Result<(), CurveError>
**Description**: 
General sanity checks on input values to ensure this is valid. These checks should be included by the validate_monotonic_* functions

### 6. validate_monotonic_increasing
**Parameters**: 
None.
**Return Type**: 
Result<(), CurveError>
**Description**: 
returns an error if there is ever x2 > x1 such that value(x2) < value(x1).

### 7. `validate_monotonic_decreasing`
**Parameters**: 
None.
**Return Type**: 
Result<(), CurveError>
**Description**: 
Validates that the curve is monotonically decreasing.

### 8. validate_complexity
**Parameter**:
max: `u32`, the maximum allowed size of the curve.
**Return Type**: 
Result<(), CurveError>
**Description**: 
returns an error if the size of the curve is more than the given max.

### 9. range
**Parameters**: 
None.
**Return Type**: 
(u128, u128)
**Description**: 
return (min, max) that can ever be returned from value. These could potentially be u128::MIN and u128::MAX.

### 10. combine_const
**Parameters**:
 const_y: `u128` value representing the y-value that will be combined with the curve.
**Return Type**: 
Curve
**Description**: 
combines a constant with a curve (shifting the curve up)

### 11. combine
**Parameter**:
other: `&Curve` value for another curve to combine with the current one.
**Return Type**: 
Curve
**Description**: 
returns a new curve that is the result of adding the given curve to this one.

### 12. `end`
**Parameters**: 
None.
**Return Type**: 
Option<u64>
**Description**: 
Returns the end point as u64 value.
