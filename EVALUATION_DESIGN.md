# SQL Boolean Expression Evaluation - Design Proposals

## Current State

The parser produces an AST with three levels:
- **BooleanExpr**: Top-level boolean expressions (AND, OR, NOT, literals, variables, relational)
- **RelationalExpr**: Comparisons that produce booleans (=, <, LIKE, BETWEEN, IN, IS NULL)
- **ValueExpr**: Value expressions (arithmetic, literals, variables)

## Design Goals

1. Variable binding (value mapping)
2. Variable validation (ensure all variables are bound)
3. Expression evaluation with proper SQL semantics
4. Type checking and coercion (SQL allows comparing different types)
5. NULL handling (SQL three-valued logic)
6. Error handling (type errors, unbound variables, division by zero, etc.)

---

## Approach 1: Visitor Pattern with Mutable Context

### Overview
Use the Visitor pattern where evaluation traverses the AST with a mutable context containing variable bindings.

### Data Structures

```rust
use std::collections::HashMap;

/// Runtime value - what variables evaluate to
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
}

/// Variable bindings
pub type ValueMapping = HashMap<String, RuntimeValue>;

/// Evaluation context
pub struct EvalContext {
    bindings: ValueMapping,
}

/// Evaluation errors
#[derive(Debug, Clone)]
pub enum EvalError {
    UnboundVariable(String),
    TypeError { expected: String, got: String },
    DivisionByZero,
    InvalidOperation(String),
}

pub type EvalResult<T> = Result<T, EvalError>;
```

### Evaluation Methods

```rust
impl BooleanExpr {
    pub fn eval(&self, ctx: &EvalContext) -> EvalResult<bool> {
        match self {
            BooleanExpr::Literal(b) => Ok(*b),
            BooleanExpr::Variable(name) => {
                match ctx.bindings.get(name) {
                    Some(RuntimeValue::Boolean(b)) => Ok(*b),
                    Some(v) => Err(EvalError::TypeError {
                        expected: "boolean".to_string(),
                        got: format!("{:?}", v),
                    }),
                    None => Err(EvalError::UnboundVariable(name.clone())),
                }
            }
            BooleanExpr::And(left, right) => {
                Ok(left.eval(ctx)? && right.eval(ctx)?)
            }
            BooleanExpr::Or(left, right) => {
                Ok(left.eval(ctx)? || right.eval(ctx)?)
            }
            BooleanExpr::Not(expr) => Ok(!expr.eval(ctx)?),
            BooleanExpr::Relational(rel) => rel.eval(ctx),
        }
    }
}

impl ValueExpr {
    pub fn eval(&self, ctx: &EvalContext) -> EvalResult<RuntimeValue> {
        match self {
            ValueExpr::Literal(lit) => Ok(lit.to_runtime_value()),
            ValueExpr::Variable(name) => {
                ctx.bindings.get(name)
                    .cloned()
                    .ok_or_else(|| EvalError::UnboundVariable(name.clone()))
            }
            ValueExpr::Add(l, r) => {
                let lv = l.eval(ctx)?;
                let rv = r.eval(ctx)?;
                RuntimeValue::add(lv, rv)
            }
            // ... other operations
        }
    }
}
```

### Pros
- Simple, straightforward implementation
- Each AST node is responsible for its own evaluation
- Easy to understand control flow
- Minimal allocation (borrows context)

### Cons
- Methods coupled to AST types (modifies ast.rs)
- Harder to swap evaluation strategies
- Testing requires creating full AST structures

---

## Approach 2: Separate Evaluator with Trait

### Overview
Create a separate `Evaluator` type that implements evaluation as a trait, keeping AST definitions clean.

### Data Structures

```rust
/// Trait for evaluable expressions
pub trait Evaluate {
    type Output;
    fn evaluate(&self, ctx: &EvalContext) -> EvalResult<Self::Output>;
}

/// Main evaluator type
pub struct Evaluator {
    context: EvalContext,
}

impl Evaluator {
    pub fn new(bindings: ValueMapping) -> Self {
        Evaluator {
            context: EvalContext { bindings }
        }
    }

    /// Validate that all variables in expression are bound
    pub fn validate(&self, expr: &BooleanExpr) -> Result<(), Vec<String>> {
        let mut unbound = Vec::new();
        self.collect_unbound_variables(expr, &mut unbound);
        if unbound.is_empty() {
            Ok(())
        } else {
            Err(unbound)
        }
    }

    /// Evaluate the expression
    pub fn eval(&self, expr: &BooleanExpr) -> EvalResult<bool> {
        expr.evaluate(&self.context)
    }
}
```

### Implementation

```rust
impl Evaluate for BooleanExpr {
    type Output = bool;

    fn evaluate(&self, ctx: &EvalContext) -> EvalResult<bool> {
        match self {
            BooleanExpr::And(l, r) => Ok(l.evaluate(ctx)? && r.evaluate(ctx)?),
            // ... rest of implementation
        }
    }
}

impl Evaluate for ValueExpr {
    type Output = RuntimeValue;

    fn evaluate(&self, ctx: &EvalContext) -> EvalResult<RuntimeValue> {
        // ... implementation
    }
}
```

### Pros
- Separation of concerns (AST vs evaluation)
- Easy to test (mock Evaluate trait)
- Can have multiple evaluation strategies
- Explicit validation step

### Cons
- More boilerplate (trait + implementations)
- Slightly more verbose

---

## Approach 3: Interpreter with Stack Machine

### Overview
Compile AST to bytecode, then evaluate using a stack machine. This is overkill for simple evaluation but provides benefits for optimization.

### Data Structures

```rust
/// Bytecode instructions
#[derive(Debug, Clone)]
pub enum Instruction {
    // Stack operations
    PushBool(bool),
    PushValue(RuntimeValue),
    LoadVar(String),

    // Boolean operations
    And,
    Or,
    Not,

    // Relational operations
    Equal,
    GreaterThan,
    LessThan,
    // ... etc

    // Value operations
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

pub struct Interpreter {
    instructions: Vec<Instruction>,
    bindings: ValueMapping,
}

impl Interpreter {
    pub fn new(bindings: ValueMapping) -> Self {
        Interpreter {
            instructions: Vec::new(),
            bindings,
        }
    }

    /// Compile AST to bytecode
    pub fn compile(&mut self, expr: &BooleanExpr) {
        self.compile_bool_expr(expr);
    }

    /// Execute bytecode
    pub fn execute(&self) -> EvalResult<bool> {
        let mut stack: Vec<RuntimeValue> = Vec::new();

        for instr in &self.instructions {
            match instr {
                Instruction::PushBool(b) => {
                    stack.push(RuntimeValue::Boolean(*b));
                }
                Instruction::And => {
                    let r = stack.pop().unwrap();
                    let l = stack.pop().unwrap();
                    // ... perform AND
                }
                // ... etc
            }
        }

        // Final stack should have one boolean
        // ... extract result
    }
}
```

### Pros
- Can optimize bytecode before execution
- Separates compilation from execution
- Can reuse compiled bytecode with different bindings
- Easier to add profiling/debugging

### Cons
- Significant complexity overhead
- More memory usage
- Overkill for simple use case
- Harder to debug

---

## Approach 4: Two-Phase (Substitution then Evaluation)

### Overview
First phase: substitute all variables with values, creating a new AST with no variables.
Second phase: evaluate the fully-concrete AST.

### Data Structures

```rust
pub struct Substituter {
    bindings: ValueMapping,
}

impl Substituter {
    /// Substitute all variables, returning error if any unbound
    pub fn substitute(&self, expr: &BooleanExpr) -> EvalResult<BooleanExpr> {
        match expr {
            BooleanExpr::Variable(name) => {
                match self.bindings.get(name) {
                    Some(RuntimeValue::Boolean(b)) => Ok(BooleanExpr::Literal(*b)),
                    Some(_) => Err(EvalError::TypeError { ... }),
                    None => Err(EvalError::UnboundVariable(name.clone())),
                }
            }
            BooleanExpr::And(l, r) => {
                Ok(BooleanExpr::And(
                    Box::new(self.substitute(l)?),
                    Box::new(self.substitute(r)?)
                ))
            }
            // ... other cases
        }
    }
}

pub struct Evaluator;

impl Evaluator {
    /// Evaluate a fully-concrete expression (no variables)
    pub fn eval(expr: &BooleanExpr) -> EvalResult<bool> {
        match expr {
            BooleanExpr::Literal(b) => Ok(*b),
            BooleanExpr::Variable(_) => {
                Err(EvalError::InvalidOperation(
                    "Unsubstituted variable".to_string()
                ))
            }
            BooleanExpr::And(l, r) => Ok(Self::eval(l)? && Self::eval(r)?),
            // ... etc
        }
    }
}

// Usage:
let substituted = substituter.substitute(&parsed_expr)?;
let result = Evaluator::eval(&substituted)?;
```

### Pros
- Clear separation of concerns
- Easy to inspect intermediate state
- Can validate substitution separately from evaluation
- Simpler evaluation logic (no context needed)

### Cons
- Allocates new AST (memory overhead)
- Two passes over the tree
- Duplicates AST structure

---

## Approach 5: Hybrid (Direct Eval with Optional Caching)

### Overview
Direct evaluation like Approach 1, but with optional caching/memoization for repeated evaluations.

### Data Structures

```rust
use std::cell::RefCell;

pub struct CachingEvaluator {
    bindings: ValueMapping,
    cache: RefCell<HashMap<*const BooleanExpr, bool>>,
}

impl CachingEvaluator {
    pub fn new(bindings: ValueMapping) -> Self {
        CachingEvaluator {
            bindings,
            cache: RefCell::new(HashMap::new()),
        }
    }

    pub fn eval(&self, expr: &BooleanExpr) -> EvalResult<bool> {
        // Check cache
        let expr_ptr = expr as *const BooleanExpr;
        if let Some(&result) = self.cache.borrow().get(&expr_ptr) {
            return Ok(result);
        }

        // Evaluate
        let result = match expr {
            BooleanExpr::And(l, r) => {
                self.eval(l)? && self.eval(r)?
            }
            // ... etc
        };

        // Cache result
        self.cache.borrow_mut().insert(expr_ptr, result);
        Ok(result)
    }
}
```

### Pros
- Optimizes repeated sub-expression evaluation
- Transparent caching
- Same API as simple evaluator

### Cons
- Unsafe pointer usage
- Interior mutability complexity
- Cache invalidation complexity
- Probably premature optimization

---

## Recommendation: Approach 2 (Separate Evaluator with Trait)

### Rationale

**Best balance of:**
- Clean separation of concerns
- Testability
- Extensibility
- Simplicity

**Implementation Plan:**

1. **Phase 1: Core Types**
   - Define `RuntimeValue` enum
   - Define `ValueMapping` type alias
   - Define `EvalError` enum
   - Define `EvalContext` struct

2. **Phase 2: Evaluation Trait**
   - Define `Evaluate` trait
   - Implement for `BooleanExpr`
   - Implement for `RelationalExpr`
   - Implement for `ValueExpr`

3. **Phase 3: Evaluator**
   - Implement `Evaluator` struct
   - Add validation method
   - Add evaluation method
   - Handle SQL NULL semantics (three-valued logic)

4. **Phase 4: SQL Semantics**
   - Type coercion rules (e.g., "123" can be compared to 123)
   - NULL propagation (NULL AND TRUE = NULL in SQL)
   - LIKE pattern matching with wildcards
   - BETWEEN inclusive range checking
   - IN list membership

5. **Phase 5: Error Handling**
   - Unbound variables
   - Type mismatches
   - Division by zero
   - Invalid operations

### Key Design Decisions

**NULL Handling Options:**

*Option A: SQL three-valued logic (TRUE/FALSE/NULL)*
```rust
pub enum SqlBool {
    True,
    False,
    Null,
}
```
Most faithful to SQL, but complex.

*Option B: Treat NULL as false (simpler)*
```rust
// NULL in comparisons yields false
```
Simpler, but not SQL-compliant.

**Recommendation:** Start with Option B, add Option A if needed.

**Type Coercion:**

SQL allows comparing different types with coercion rules:
- String to number: "123" → 123
- Number to string: 123 → "123"
- Boolean to number: TRUE → 1, FALSE → 0

**Recommendation:** Implement strict typing first, add coercion as needed.

### Example Usage

```rust
use std::collections::HashMap;
use sqlexpr_rust::{parse, Evaluator, RuntimeValue};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse expression
    let expr = parse("x > 5 AND status = 'active'")?;

    // Create value mapping
    let mut bindings = HashMap::new();
    bindings.insert("x".to_string(), RuntimeValue::Integer(10));
    bindings.insert("status".to_string(), RuntimeValue::String("active".to_string()));

    // Create evaluator
    let evaluator = Evaluator::new(bindings);

    // Validate all variables are bound
    evaluator.validate(&expr)?;

    // Evaluate
    let result = evaluator.eval(&expr)?;
    println!("Result: {}", result); // true

    Ok(())
}
```

---

## Alternative: Approach 1 for Simplicity

If simplicity is paramount and we don't mind coupling evaluation to AST:

### Minimal Implementation

```rust
// Add to ast.rs
impl BooleanExpr {
    pub fn eval(&self, bindings: &HashMap<String, RuntimeValue>) -> Result<bool, String> {
        // Simple, direct evaluation
    }
}
```

**Pros:** Minimal code, easy to understand
**Cons:** Couples AST to evaluation, harder to test in isolation

---

## Comparison Matrix

| Approach | Complexity | Separation | Performance | Testability | Extensibility |
|----------|-----------|------------|-------------|-------------|---------------|
| 1. Visitor | Low | Poor | Excellent | Medium | Poor |
| 2. Trait | Medium | Excellent | Excellent | Excellent | Excellent |
| 3. Stack Machine | High | Excellent | Good | Medium | Excellent |
| 4. Substitution | Medium | Good | Poor | Good | Medium |
| 5. Caching | High | Medium | Excellent* | Medium | Medium |

*Only for repeated evaluations

---

## Next Steps

If Approach 2 is selected:

1. Create `src/eval.rs` module
2. Define runtime types and errors
3. Implement `Evaluate` trait
4. Add comprehensive tests
5. Document SQL semantics (especially NULL handling)
6. Add examples showing usage

The implementation should be incremental:
- Start with literals only
- Add variables
- Add boolean operators
- Add relational operators
- Add value expressions
- Add SQL-specific features (LIKE, BETWEEN, IN, NULL)
