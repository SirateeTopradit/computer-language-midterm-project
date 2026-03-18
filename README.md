# SRISC Compiler Midterm Project

## Overview

computer-language-midterm-project

### SRISC Language Specification

SRISC supports a subset of RISC-V instructions:
- **ALU Operations**: `add`, `sub`, `and`, `or`
- **Memory Operations**: `ld`, `sd`, `lw`, `sw`
- **Branch Operations**: `beq`, `bne`, `blt`, `bge`
- **Registers**: `x0` through `x31`
- **Labels**: `L0` through `L10`
- **Keywords**: `.code` (start of program), `.end` (end of program)

### LL(1) Grammar

The parser implements the following context-free grammar, refactored for LL(1) compatibility:

```text
<Program>     -> .code <StmtList> .end
<StmtList>    -> <Stmt> <StmtList> | ε
<Stmt>        -> LABEL : | <Instruction>
<Instruction> -> ALU_OP REG , REG , <Operand>
               | MEM_OP REG , IMM ( REG )
               | BR_OP REG , REG , LABEL
<Operand>     -> REG | IMM
```

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)

### Running the Parser

The parser reads from `stdin` and outputs `yes` for valid syntax or `no` for any errors.

```powershell
# Using echo to pipe code
echo ".code add x1, x2, x3 .end" | cargo run

# Reading from a file
Get-Content source.srisc | cargo run
```

### Running Tests

```powershell
cargo test
```