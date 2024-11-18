## lexer 
主要将 sql 解析称 Token， Token 主要可以分成以下几类：
1. Number(String): 数字类型，int 和 float 都在里面
2. String: 字符串常量，这个应该会被 "" 包裹
3. Ident(String): 可以是 table name，database name 等其他 sql 中的定义
4. Keyword(String): sql 的一次关键字，比如说 create table ， select，drop 等
5. 一些关系运算符

## parser
将 token 转换成 ast expression 

## ast 
parser 的输出结果，结构主要可以分成以下几类：
1. Literal: 这里包括 bool，int，float，string，Null 
2. Operator： 运算符号， 比如加减乘除等关系运算符
3. Function： 自定义的一些函数，比如 sqrt 这种
4. All: 所有的 column 

pub enum Expression {
    All,
    Literal(Literal),
    Operator(Operator),
    /// A function call (name and parameters).
    Function(FunctionName, Vec<Expression>),
}