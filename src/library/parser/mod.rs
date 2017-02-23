use library::lexeme::Type::*;
use library::lexeme::Token;

#[derive(Debug)]
struct SymbolTable {
    typ: i32,
    id_name: String,
    is_assigned: bool,
    assigned_val: String,
}

impl Clone for SymbolTable {
    fn clone(&self) -> SymbolTable {
        let id = self.id_name.clone();
        let val = self.assigned_val.clone();
        SymbolTable {
            assigned_val: val,
            id_name: id,
            ..*self
        }
    }
}

static mut IN_BLOCK_STMNT: bool = false;
static mut IN_EXPR: bool = false;

/**
 * parse_program:
 * parse c program from Token Vector
 * return String Vector of Rust equivalent code
 * supports:
 * - method
 * - variable declaration
 * - variable assignment
 * - if
 * - comments
 */
pub fn parse_program(lexeme: &Vec<Token>) -> Vec<String> {

    let mut stream: Vec<String> = Vec::new();
    let mut head: usize = 0;
    let mut lookahead: usize = 0;
    let mut temp_lexeme: Vec<Token> = Vec::new();

    while head < lexeme.len() {
        // gets both base type and token type
        match lexeme[head].get_type() {

            // matches any datatype
            (BASE_DATATYPE, _) => {
                lookahead += 2;
                match lexeme[lookahead].get_token_type() {

                    // function
                    LEFT_BRACKET => {
                        //inside the function
                        unsafe {
                            IN_BLOCK_STMNT = true;
                           }

                        while lexeme[lookahead].get_token_type() != LEFT_CBRACE {
                            lookahead += 1;
                        }
                        // advance lookahead to end of block
                        lookahead = skip_block(&lexeme, lookahead + 1);
                        // collect entire function block
                        while head < lookahead {
                            let l: Token = lexeme[head].clone();
                            temp_lexeme.push(l);
                            head += 1;
                        }

                        //parse function block
                        stream.append(&mut parse_function(&temp_lexeme));
                        unsafe {
                            IN_BLOCK_STMNT = false;
                        }

                        temp_lexeme.clear();
                    }
                    //array declaration found
                    LEFT_SBRACKET => {
                         lookahead = skip_stmt(&lexeme, lookahead);

                        // collect variable declaration
                        while head != lookahead {
                            let l: Token = lexeme[head].clone();
                            temp_lexeme.push(l);
                            head += 1;
                        }
                        // parse declaration
                        stream.append(&mut parse_array_declaration(&temp_lexeme));
                        temp_lexeme.clear();  
                    },
                    // variable declaration or declaration + assignment
                    SEMICOLON | COMMA | OP_ASSIGN => {
                        lookahead = skip_stmt(&lexeme, lookahead);

                        // collect variable declaration
                        while head != lookahead {
                            let l: Token = lexeme[head].clone();
                            temp_lexeme.push(l);
                            head += 1;
                        }
                        // parse declaration
                        stream.append(&mut parse_declaration(&temp_lexeme));
                        temp_lexeme.clear();
                    }


                    _ => {}
                };
            }

            // matches if statement
            (_, KEYWORD_IF) => {
                let mut temp_lexeme: Vec<Token> = Vec::new();

                // move lookahead past conditon
                while lexeme[lookahead].get_token_type() != RIGHT_BRACKET {
                    lookahead += 1;
                }
                lookahead += 1;

                // move lookahead past block
                if lexeme[lookahead].get_token_type() == LEFT_CBRACE {
                    lookahead = skip_block(&lexeme, lookahead + 1);
                }
                // move lookahead past block for 'if' without braces
                else {
                    lookahead = skip_stmt(&lexeme, lookahead);
                }
                // collect if block
                while head < lookahead {
                    let l: Token = lexeme[head].clone();
                    temp_lexeme.push(l);
                    head += 1;
                }

                // parse if
                stream.append(&mut parse_if(&temp_lexeme));
            }

            (_, KEYWORD_ELSE) => {
                let mut temp_lexeme: Vec<Token> = Vec::new();
                let mut elseif: bool = false;

                head += 1;
                lookahead = head;
                if lexeme[head].get_token_type() == KEYWORD_IF {
                    elseif = true;
                }

                if lexeme[lookahead].get_token_type() == LEFT_CBRACE {
                    lookahead = skip_block(&lexeme, lookahead + 1);
                }

                else {
                    lookahead = skip_stmt(&lexeme, lookahead);
                }

                while head < lookahead {
                    let l: Token = lexeme[head].clone();
                    temp_lexeme.push(l);
                    head += 1;
                }
                //** parse else body
                stream.push("else".to_string());
                if elseif == false {
                    stream.push("{".to_string());
                }
                stream.append(&mut parse_program(&temp_lexeme));
                if elseif == false {
                    stream.push("}".to_string());
                }
            }

            (_, KEYWORD_SWITCH) => {
                let mut temp_lexeme: Vec<Token> = Vec::new();
                head += 2;
                lookahead = head;
                stream.push("match".to_string());

                // find starting of switch block
                while lexeme[lookahead].get_token_type() != LEFT_CBRACE {
                    lookahead += 1;
                }

                // move back to find the variable/result to be matched
                lookahead -= 1;
                // single variable
                if lookahead - head == 1 {
                    stream.push(lexeme[lookahead-1].get_token_value());

                }
                // expression
                else {
                    while head < lookahead {
                        let l: Token = lexeme[head].clone();
                        temp_lexeme.push(l);
                        head += 1;
                    }
                    stream.append(&mut parse_program(&temp_lexeme));
                    temp_lexeme.clear();
                }

                // move forward to the starting of the block
                head += 3;
                stream.push("{".to_string());
                

                //head is at case
                lookahead = skip_block(&lexeme, head);
                while head < lookahead {
                    let l: Token = lexeme[head].clone();
                    temp_lexeme.push(l);
                    head += 1;
                }

                stream.append(&mut parse_case(&temp_lexeme));
                stream.push("}".to_string());
            }

            (_, KEYWORD_WHILE) => {
                let mut temp_lexeme: Vec<Token> = Vec::new();

                // move lookahead past conditon
                while lexeme[lookahead].get_token_type() != RIGHT_BRACKET {
                    lookahead += 1;
                }
                lookahead += 1;

                // move lookahead past block
                if lexeme[lookahead].get_token_type() == LEFT_CBRACE {
                    lookahead = skip_block(&lexeme, lookahead + 1);
                }
                // move lookahead past block for 'if' without braces
                else {
                    lookahead = skip_stmt(&lexeme, lookahead);
                }
                // collect if block
                while head < lookahead {
                    let l: Token = lexeme[head].clone();
                    temp_lexeme.push(l);
                    head += 1;
                }

                // parse if
                stream.append(&mut parse_while(&temp_lexeme));
            }

            // matches do while statement
            (_, KEYWORD_DO) => {
                let mut temp_lexeme: Vec<Token> = Vec::new();

                // move lookahead past block
                lookahead = skip_block(&lexeme, lookahead+2);
                lookahead = skip_stmt(&lexeme, lookahead);

                // collect while block
                while head < lookahead {
                    let l: Token = lexeme[head].clone();
                    temp_lexeme.push(l);
                    head += 1;
                }
                // parse while
                stream.append(&mut parse_dowhile(&temp_lexeme));
            }

            // matches for statement
            (_, KEYWORD_FOR) => {
                let mut temp_lexeme: Vec<Token> = Vec::new();

                while lexeme[lookahead].get_token_type() != LEFT_CBRACE {
                    lookahead += 1;
                }
                lookahead += 1;
                lookahead = skip_block(&lexeme, lookahead);

                while head < lookahead {
                    let l: Token = lexeme[head].clone();
                    temp_lexeme.push(l);
                    head += 1;
                }

                // print_lexemes(&temp_lexeme, 0, temp_lexeme.len());
                stream.append(&mut parse_for(&temp_lexeme));
            }

            // matches single and multi-line comment
            (BASE_COMMENT, _) => {
                stream.push(lexeme[head].get_token_value() + "\n");
                head += 1;
                lookahead = head;
            }

            // assignment statements
            (_, IDENTIFIER) => {
                let mut temp_lexeme: Vec<Token> = Vec::new();
                //identifier = expr
                //identifier()
                //identifier+expr
                //identifier OP_INC|OP_DEC; =>postfix


                match lexeme[head + 1].get_type() {
                    (_, OP_ASSIGN) => {
                        // move lookahead past statement
                        if lexeme[head + 3].get_token_type() == COMMA {
                            lookahead = head + 3;
                            while head < lookahead + 1 {
                                let l: Token = lexeme[head].clone();
                                temp_lexeme.push(l);
                                head += 1;
                            }
                            stream.append(&mut parse_assignment(&temp_lexeme));
                            temp_lexeme.clear();
                        } else {
                            lookahead = skip_stmt(&lexeme, lookahead);
                            // collect statement
                            while head < lookahead {
                                let l: Token = lexeme[head].clone();
                                temp_lexeme.push(l);
                                head += 1;
                            }

                            // parse assignment
                            stream.append(&mut parse_assignment(&temp_lexeme));
                        }
                    }
                    (BASE_UNOP, _) => unsafe {
                        if IN_EXPR != true {
                            stream.push(lexeme[head].get_token_value());
                            stream.push(match lexeme[head + 1].get_token_type() {
                                OP_INC => "+=1".to_string(),
                                OP_DEC => "-=1".to_string(),
                                _ => " ;".to_string(),
                            });
                            head += 2;
                        } else {
                            head += 2;
                        }
                    },
                    (BASE_BINOP, _) => {
                        // move lookahead past statement
                        lookahead = skip_stmt(&lexeme, lookahead);
                        // collect statement
                        while head < lookahead {
                            let l: Token = lexeme[head].clone();
                            temp_lexeme.push(l);
                            head += 1;
                        }

                        // parse assignment
                        stream.append(&mut parse_expr(&temp_lexeme));

                    }
                    (_,LEFT_BRACKET) =>{
                        while lexeme [head].get_token_type()!= RIGHT_BRACKET {
                            stream.push(lexeme[head].get_token_value());
                            head+=1;
                        }
                        stream.push(lexeme[head].get_token_value());
                        head+=1;
                    },
                    (_, _) => {
                        if lexeme[head].get_token_type() != RIGHT_CBRACE {
                              stream.push(lexeme[head].get_token_value());
                        }
                        head += 1;
                        lookahead = head;
                    }
                };
            }
            (BASE_UNOP, _) => {
                stream.push(lexeme[head + 1].get_token_value());
                stream.push(match lexeme[head].get_token_type() {
                    OP_INC => "+=1".to_string(),
                    OP_DEC => "-=1".to_string(),
                    _ => " ;".to_string(),
                });
                head += 2;
            }
            // if all fails
            (_, _) => {
                if lexeme[head].get_token_type() != RIGHT_CBRACE {
                    if lexeme[head].get_token_type() == COMMA{ 
                        println!("problem");
                         stream.push(";".to_string()); }else{     
                    stream.push(lexeme[head].get_token_value());
                    }
                }
                head += 1;
                lookahead = head;
            }

        };
    }
    //return the rust lexeme to main
    stream
}


/**
 * print_lexemes:
 * prints the lexemes in the lexeme vector
 * from index start to end
 */
fn print_lexemes(lexeme: &Vec<Token>, start: usize, end: usize) {
    println!("----------lexeme-start------------");
    for i in start..end {
        println!("{}> {}", i, lexeme[i].get_token_value());
    }
    println!("----------lexeme-end------------");
}


/**
 * skip_stmt:
 * forwards the lookahead by one statement
 * returns the lookahead at the lexeme after the semi-colon
 */
fn skip_stmt(lexeme: &Vec<Token>, mut lookahead: usize) -> usize {
    while lexeme[lookahead].get_token_type() != SEMICOLON {
        lookahead += 1;
    }
    lookahead + 1
}



/**
 * skip_block:
 * forwards the lookahead by one block
 * returns the lookahead at the lexeme after the closing brace
 */
fn skip_block(lexeme: &Vec<Token>, mut lookahead: usize) -> usize {
    let mut paren = 1;

    // while all braces are not closed
    // skip nested blocks if any
    while paren != 0 && lookahead < lexeme.len() {
        if lexeme[lookahead].get_token_type() == LEFT_CBRACE {
            paren += 1;
        }
        if lexeme[lookahead].get_token_type() == RIGHT_CBRACE {
            paren -= 1;
        }
        lookahead += 1;
    }
    lookahead
}


/**
 * parse_function:
 * parse c/c++ function into rust equivalent function
 */
fn parse_function(lexeme: &Vec<Token>) -> Vec<String> {
    let mut temp_lexeme: Vec<Token> = Vec::new();
    let mut head: usize = 3;
    let mut lookahead: usize = head;
    let mut stream: Vec<String> = Vec::new();

    stream.push("fn".to_string());
    stream.push(lexeme[1].get_token_value());
    stream.push("(".to_string());

    // parse arguments differenly for functions that are not main
    // since rust does not have arguments or return type for main
    if lexeme[1].get_token_type() != MAIN {

        // collect arguments
        while lexeme[lookahead].get_token_type() != RIGHT_BRACKET {
            lookahead += 1;
        }
        while head < lookahead {
            let l: Token = lexeme[head].clone();
            temp_lexeme.push(l);
            head += 1;
        }
        // parse arguments
        stream.append(&mut parse_arguments(&temp_lexeme));
        temp_lexeme.clear();

        stream.push(")".to_string());
        
        // parse return type
        if let Some(rust_type) = parse_type(lexeme[0].get_token_type() as i32) {
            if rust_type != "void" {
                stream.push("->".to_string());
                stream.push(rust_type);
            }
        }

        stream.push("{".to_string());
    }
    // declare argc and argv inside main, if required
    else {
        stream.push(")".to_string());
        stream.push("{".to_string());
        if lexeme[head].get_token_type() != RIGHT_BRACKET {
            stream.push("let mut argv = env::args();".to_string());
            stream.push("let mut argc = argv.len();".to_string());
        }
    }

    while lexeme[head].get_token_type() != LEFT_CBRACE {
        head += 1
    }
    head += 1;

    // collect function body
    // len - 1  so that '}' is excluded
    while head < lexeme.len() - 1 {
        let l: Token = lexeme[head].clone();
        temp_lexeme.push(l);
        head += 1;
    }
    // parse function body
    stream.append(&mut parse_program(&temp_lexeme));
    stream.push("}".to_string());
    stream
}


/**
 * parse-arguments:
 * parse c/c++ formal arguments in the function signature
 * into rust equivalent arguments
 */
fn parse_arguments(lexeme: &Vec<Token>) -> Vec<String> {
    let mut stream: Vec<String> = Vec::new();
    let mut head: usize = 0;
    while head < lexeme.len() {
        if lexeme[head].get_token_type() == COMMA {
            stream.push(",".to_string());
            head += 1;
            continue;
        }
        // push identifier
        stream.push(lexeme[head + 1].get_token_value()); //int f(int val)
        stream.push(":".to_string());

        // parse argument type
        if let Some(rust_type) = parse_type(lexeme[head].get_token_type() as i32) {
            stream.push(rust_type);
        }
        head += 2;
    }
    stream
}


/**
 * parse_declaration:
 * parse c/c++ declaration into rust
 * equivalent statements
 */
fn parse_declaration(lexeme: &Vec<Token>) -> Vec<String> {
       let mut stream: Vec<String> = Vec::new();
 
    let mut sym_tab: Vec<SymbolTable> = Vec::new();
    let mut sym: SymbolTable = SymbolTable {
        typ: -1,
        id_name: "undefined_var".to_string(),
        is_assigned: false,
        assigned_val: "NONE".to_string(),
    };
    let mut head: usize = 1;
    //let sym_idx:usize=0;
    while head < lexeme.len() {

        match lexeme[head].get_token_type() {

            IDENTIFIER => sym.id_name = lexeme[head].get_token_value(),

            OP_ASSIGN => {
                head += 1;
                sym.assigned_val = lexeme[head].get_token_value();
                sym.is_assigned = true;
            }

            SEMICOLON | COMMA => {
                // used enum value in the symbol table
                sym.typ = lexeme[0].get_token_type() as i32;
                sym_tab.push(sym.clone());
            }
            LEFT_SBRACKET =>{
                let mut temp_lexeme : Vec<Token> = Vec::new();

                temp_lexeme.push(lexeme[0].clone());
               //move to next
               let mut m = head-1;
              while lexeme[m].get_token_type() != SEMICOLON{
                temp_lexeme.push(lexeme[m].clone());
                m+=1;
            }
              temp_lexeme.push(lexeme[m].clone());
             stream.append(&mut parse_array_declaration(&temp_lexeme));
            
            while lexeme[head].get_token_type() != RIGHT_SBRACKET { head+=1;}
            head=m;
            }
            _ => {}
        };
        head += 1;

    }

    for i in &sym_tab {

        // get identifier
        //for declaration out of any blocks(global)
        unsafe {
            if IN_BLOCK_STMNT == false {
                stream.push("static".to_string());
            } else {
                stream.push("let mut".to_string());
            }
        }
        stream.push(i.id_name.clone());
        stream.push(":".to_string());
        // get the rust type
        if let Some(rust_type) = parse_type(i.typ) {
            stream.push(rust_type);
        } else {
            stream.push("UNKNOWN_TYPE".to_string());
        }

        // take care of assignment
        if i.is_assigned {
            stream.push("=".to_string());
            stream.push((&i.assigned_val).to_string());
        }
        stream.push(";".to_string());
    }
    stream
}


/**
 * parse_if:
 * parse c/c++ if statements into rust
 * equivalent statements
 */
fn parse_if(lexeme: &Vec<Token>) -> Vec<String> {
    let mut stream: Vec<String> = Vec::new();
    let mut head: usize = 0;

    stream.push("if".to_string());
    head += 1;

    //skip '('
    head += 1;

    // condition
    while lexeme[head].get_token_type() != RIGHT_BRACKET {
        stream.push(lexeme[head].get_token_value());
        head += 1;
    }
    head += 1;

    stream.push("{".to_string());

    if lexeme[head].get_token_type() == LEFT_CBRACE {
        head += 1;
    }

    // collect if body
    let mut temp_lexeme: Vec<Token> = Vec::new();
    while head < lexeme.len() {
        let l: Token = lexeme[head].clone();
        temp_lexeme.push(l);
        head += 1;
    }
    // parse if body
    stream.append(&mut parse_program(&temp_lexeme));

    stream.push("}".to_string());
    stream
}


/**
 * parse_while:
 * parse c/c++ while statements into rust
 * equivalent statements
 */
fn parse_while(lexeme: &Vec<Token>) -> Vec<String> {
    let mut stream: Vec<String> = Vec::new();
    let mut head: usize = 0;
    let mut no_cond = false;
   // stream.push("while".to_string());
    head += 1;

    //skip '('
    head += 1;
    let lhead = head;
    // condition
    let mut cond_stream : Vec<String> = Vec::new();
    while lexeme[head].get_token_type() != RIGHT_BRACKET {
        cond_stream.push(lexeme[head].get_token_value());
        head += 1;
    }
     if head == lhead { no_cond = true; }
    head += 1;
   
    if lexeme[head].get_token_type() == LEFT_CBRACE {
        head += 1;
    }

    // collect while body
    let mut temp_lexeme: Vec<Token> = Vec::new();
    while head < lexeme.len() {
        let l: Token = lexeme[head].clone();
        temp_lexeme.push(l);
        head += 1;
    }
    // parse while body
    let mut body_stream = &mut parse_program(&temp_lexeme);

    if no_cond == true { stream.push("loop".to_string()); }else {
        stream.push("while".to_string());
        stream.append(&mut cond_stream);
    }
    stream.push("{".to_string());
    stream.append(&mut body_stream);

    stream.push("}".to_string());
    stream
}


/**
 * parse_dowhile:
 * parse c/c++ do while statements into rust
 * equivalent statements
 */
fn parse_dowhile(lexeme: &Vec<Token>) -> Vec<String> {
    let mut stream: Vec<String> = Vec::new();
    let mut head: usize = 0;
    let mut lookahead: usize;

    stream.push("while".to_string());
    stream.push("{".to_string());
    // println!("{}", lexeme[head].get_token_value());
    head += 2;
    lookahead = head;
    
    lookahead = skip_block(&lexeme, lookahead) - 1;
    // collect while body
    let mut temp_lexeme: Vec<Token> = Vec::new();
    while head < lookahead {
        let l: Token = lexeme[head].clone();
        temp_lexeme.push(l);
        head += 1;
    }
    // parse while body
    
    stream.append(&mut parse_program(&temp_lexeme));
    temp_lexeme.clear();

    head += 3;
    // print_lexemes(&lexeme, head, lexeme.len());
    while lexeme[head].get_token_type() != RIGHT_BRACKET {
        stream.push(lexeme[head].get_token_value());
        head+=1;
    }

    stream.push("}".to_string());
    stream.push("{".to_string());
    stream.push("}".to_string());
    stream.push(";".to_string());
    stream
}


fn parse_case(lexeme: &Vec<Token>) -> Vec<String> {
    let mut stream: Vec<String> = Vec::new();
    //head is at case
    let mut head: usize = 0;
    let mut lookahead: usize;
    let mut temp_lexeme: Vec<Token> = Vec::new();
    let mut def: bool = false;

    while head < lexeme.len() {
        if lexeme[head].get_token_type() == KEYWORD_DEFAULT {
            stream.push("_".to_string());
            def = true;
        }
        else {
            head += 1; //head is at matching value
            stream.push(lexeme[head].get_token_value());
        }

        head += 1; // head is at :
        stream.push("=>".to_string());

        // either brace or no brace
        head += 1;
        if lexeme[head].get_token_type() == LEFT_CBRACE {
            head += 1;
            lookahead = skip_block(&lexeme, head) - 1;
        }
        else {
            lookahead = head;
            let mut tok_type = lexeme[lookahead].get_token_type();
            while tok_type != KEYWORD_CASE && tok_type != KEYWORD_DEFAULT && tok_type != RIGHT_CBRACE {
                lookahead += 1;
                tok_type = lexeme[lookahead].get_token_type();
            }
        }
        while head < lookahead {
            let l: Token = lexeme[head].clone();
            temp_lexeme.push(l);
            head += 1;
        }
        stream.push("{".to_string());
        stream.append(&mut parse_program(&temp_lexeme));
        stream.push("}".to_string());

        
        if head < lexeme.len() && lexeme[head].get_token_type() == RIGHT_CBRACE {
            head += 1;
        }
        temp_lexeme.clear();
    }
    if def == false {
        stream.push("_".to_string());
        stream.push("=>".to_string());
        stream.push("{".to_string());
        stream.push("}".to_string());
    }
    stream
}


/**
 * parse_for:
 * parse c/c++ do while statements into rust
 * equivalent statements
 *
 * Identify infinite loops and replace for with loop{} 
 */
fn parse_for(lexeme: &Vec<Token>) -> Vec<String> {
    let mut stream: Vec<String> = Vec::new();
    let mut head: usize = 0;
    let mut lookahead: usize;
    let mut temp_lexeme: Vec<Token> = Vec::new();
  
    while lexeme[head].get_token_type() != LEFT_BRACKET {
        head += 1;
    }
    head += 1;
    lookahead = head;
    // stream.push("while 1 == 1 {}".to_string());
    
    //for (int i =0; )
    let decl:bool =  if lexeme[head].get_base_type() == BASE_DATATYPE { true } else {false };
    // let mut no_init:bool; //no initialization
    let mut no_cond:bool = false; //if no condition to terminate
    let mut no_updation:bool = false; //no inc/dec of loop counter



    let mut body:Vec<String> = Vec::new();
    let mut updation:Vec<String> = Vec::new();
    let mut term_cond:Vec<String> = Vec::new();        
    // initial assignment
    lookahead = skip_stmt(&lexeme, lookahead);
    
    //incase of initialization expressio for (;i<10;i++) ; common case
    if head+1 < lookahead {
        while head < lookahead {
            let l: Token = lexeme[head].clone();
            temp_lexeme.push(l);
            head += 1;
        }
        
        if decl == true {
            stream.append(&mut parse_declaration(&temp_lexeme));    
        }
        else {
            stream.append(&mut parse_assignment(&temp_lexeme));
        }
    }
    else {
        head+=1;
        // no_init = true;
    }
    temp_lexeme.clear();
    
    // println!("Initial Assignment {:?}", stream);
    //stream.push("while".to_string());

    // terminating condition
    lookahead = skip_stmt(&lexeme, lookahead);

  
    if head+1 < lookahead {
        while head < lookahead-1 {
            term_cond.push(lexeme[head].get_token_value());
         head += 1;
         }
    }else{ no_cond = true ;}
    head+=1;
    temp_lexeme.clear();
    // println!("Terminating Condition {:?}", stream);
  
    lookahead = head;
    // update expression
    while lexeme[lookahead].get_token_type() != RIGHT_BRACKET {
        let l: Token = lexeme[lookahead].clone();
        temp_lexeme.push(l);
        lookahead += 1;
    }
    //no_updation
    if head == lookahead{ 
        no_updation = true;
    }else{

    temp_lexeme.push(Token::new(String::from(";"), BASE_NONE, SEMICOLON, 0, 0));
    updation.append(&mut parse_program(&temp_lexeme));
    temp_lexeme.clear();
    }
    head = lookahead;
    head += 1;
    if lexeme[head].get_token_type() == LEFT_CBRACE {
        head += 1;
        lookahead = skip_block(&lexeme, head);
    }
    else {
        lookahead = skip_stmt(&lexeme, head);
    }
    // println!("Update Expression {:?}", temp_stream);

    // lookahead = skip_block(&lexeme, lookahead);
    while head < lookahead {
        let l: Token = lexeme[head].clone();
        temp_lexeme.push(l);
        head += 1;
    }
    body.append(&mut parse_program(&temp_lexeme));

    if no_cond == true {
        stream.push("\nloop".to_string());
    }else{
        stream.push("\nwhile".to_string());
        stream.append(&mut term_cond); //append termianating condition
    }
    stream.push("{".to_string());
    stream.append(&mut body);
    if no_updation != true {
        stream.append(&mut updation);
    }
  
    stream.push("}".to_string());
    
    stream
}


/**
 * fn parse_type:
 * takes the integer value of type Type
 * returns either the equivalent Rust type as a string or
 * None, if does not correspond to any c/c++ type
 */
fn parse_type(c_type: i32) -> Option<String> {
    match c_type {
        0 => Some("i32".to_string()),
        1 => Some("i16".to_string()),
        2 => Some("i64".to_string()),
        3 => Some("f32".to_string()),
        4 => Some("f64".to_string()),
        5 => Some("char".to_string()),
        6 => Some("bool".to_string()),
        7 => Some("void".to_string()),
        _ => None,
    }
}


/* parse_assignment:
 * parse c/c++ assignment statements into rust equivalent code
 * compound assignments must be converted to declarations
 * as rust doesnt support compound assignment
 */
fn parse_assignment(lexeme: &Vec<Token>) -> Vec<String> {
    let mut stream: Vec<String> = Vec::new();
    // let mut lookahead = lexeme.len();
    let mut thead: usize = 2;
    let mut lexeme1: Vec<Token> = Vec::new();

    let n = 2;
    let m = 3;

    let mut tstream: Vec<String> = Vec::new();

    if lexeme[n].get_base_type() == BASE_UNOP {

        while lexeme[thead].get_token_type() != SEMICOLON {
            lexeme1.push(lexeme[thead].clone());
            thead += 1;
        }
        lexeme1.push(lexeme[thead].clone());
        stream.push(lexeme[0].get_token_value());
        stream.push(lexeme[1].get_token_value());
        stream.append(&mut parse_expr(&lexeme1));
    } else if lexeme[m].get_base_type() == BASE_UNOP {

        while lexeme[thead].get_token_type() != SEMICOLON {
            lexeme1.push(lexeme[thead].clone());
            thead += 1;
        }
        lexeme1.push(lexeme[thead].clone());
        stream.push(lexeme[0].get_token_value());
        stream.push(lexeme[1].get_token_value());
        stream.append(&mut parse_expr(&lexeme1));
    } else if lexeme[m].get_base_type() == BASE_BINOP {
         while lexeme[thead].get_token_type() != SEMICOLON {
            lexeme1.push(lexeme[thead].clone());
            thead += 1;
        }
        lexeme1.push(lexeme[thead].clone());
        stream.push(lexeme[0].get_token_value());
        stream.push(lexeme[1].get_token_value());
        stream.append(&mut parse_expr(&lexeme1));
    } else {
        if lexeme[m].get_token_type() != SEMICOLON && lexeme[m].get_token_type() != COMMA {
            while lexeme[thead].get_token_type() != SEMICOLON &&
                  lexeme[thead].get_token_type() != COMMA {
                lexeme1.push(lexeme[thead].clone());
                thead += 1;
            }
            lexeme1.push(lexeme[thead].clone());
            stream.append(&mut parse_program(&lexeme1));
         }
        stream.push(lexeme[0].get_token_value());
        stream.push(lexeme[1].get_token_value());
        if lexeme[n].get_base_type() == BASE_UNOP {
            stream.push(lexeme[m].get_token_value());
        } else {
            stream.push(lexeme[n].get_token_value());
        }
        stream.push(";".to_string());

    }
    if tstream.len() > 0 {
        stream.append(&mut tstream);
    }
    stream
}


/* parse_assignment:
 * parse c/c++ expression statements into rust equivalent code
 */
fn parse_expr(lexeme: &Vec<Token>) -> Vec<String> {
    let mut stream: Vec<String> = Vec::new();
    // let mut lookahead = lexeme.len();
    let mut tstream: Vec<String> = Vec::new();
    let mut thead: usize = 0;

    let mut prev_id = " ".to_string();
    let mut typ = OTHER;
    //a=b+c++;

    while lexeme[thead].get_token_type() != SEMICOLON {

        if lexeme[thead].get_base_type() == BASE_UNOP {
            //incase of post
            if typ == IDENTIFIER {
                tstream.push(prev_id.clone());
                tstream.push(match lexeme[thead].get_token_type() {
                    OP_INC => "+=1".to_string(),
                    OP_DEC => "-=1".to_string(),
                    _ => " ;".to_string(),
                });
                tstream.push(";".to_string());

                thead + 1;
                //continue;
            }
            // incase of pre
            else {
                stream.push("(".to_string());
                stream.push(lexeme[thead + 1].get_token_value());
                stream.push(match lexeme[thead].get_token_type() {
                    OP_INC => "+=1".to_string(),
                    OP_DEC => "-=1".to_string(),
                    _ => " ;".to_string(),
                });
                stream.push(")".to_string());
                thead += 1;
            }


        } else {

            if lexeme[thead].get_base_type() != BASE_UNOP {
                stream.push(lexeme[thead].get_token_value());
            }
        }

        typ = lexeme[thead].get_token_type();
        prev_id = lexeme[thead].get_token_value();

        thead += 1;
    }
    stream.push(";".to_string());
    if tstream.len() > 0 {
        stream.append(&mut tstream);
    }
    stream
}


fn parse_array_declaration(lexeme: &Vec<Token>) -> Vec<String> {
    let mut stream: Vec<String> = Vec::new();
     let mut typ:String=" ".to_string();

    //int a[10];
    if let Some(t) = parse_type(lexeme[0].get_token_type() as i32) { typ = t ;}
        stream.push("let mut ".to_string());
    let mut head =0;
    stream.push(lexeme[head+1].get_token_value());
    stream.push(":[".to_string()+&typ[..]+";"+&lexeme[head+3].get_token_value()[..]+"]");
    head = 5;
    let mut lookahead=head;
    while lexeme[lookahead].get_token_type() != SEMICOLON{
     lookahead+=1;
    }
    let mut temp_lexeme:Vec<Token> = Vec::new();
    if lexeme[head].get_token_type() == COMMA {
            temp_lexeme.push(lexeme[0].clone());
            //move to next
            head+=1;
            while lexeme[head].get_token_type() != SEMICOLON{
                temp_lexeme.push(lexeme[head].clone());
                head+=1;
            }
            stream.push("; ".to_string());
             temp_lexeme.push(lexeme[head].clone());
            stream.append(&mut parse_program(&temp_lexeme))
    }else   if lexeme[head].get_token_type() ==OP_ASSIGN {
        while lexeme[head].get_token_type() != SEMICOLON && lexeme[head].get_token_type() !=RIGHT_CBRACE {
           stream.push( match lexeme[head].get_token_type(){
                             LEFT_CBRACE => "[".to_string(),
                            
                            _ => lexeme[head].get_token_value(),
           });
           head+=1;
   }
   stream.push("]; ".to_string());
   }else{
       stream.push("; ".to_string());
   }
   
    stream
}


#[test]
fn test_parse_if_braces() {
    let tok_vector = vec![Token::new(String::from("if"), BASE_NONE, KEYWORD_IF, 0, 0),
                          Token::new(String::from("("), BASE_NONE, LEFT_BRACKET, 0, 1),
                          Token::new(String::from("a"), BASE_NONE, IDENTIFIER, 0, 2),
                          Token::new(String::from("=="), BASE_BINOP, OP_EQU, 0, 3),
                          Token::new(String::from("a"), BASE_NONE, IDENTIFIER, 0, 4),
                          Token::new(String::from(")"), BASE_NONE, RIGHT_BRACKET, 0, 5),
                          Token::new(String::from("{"), BASE_NONE, LEFT_CBRACE, 0, 6),
                          Token::new(String::from("/*Do something here*/"),
                                     BASE_COMMENT,
                                     COMMENT_MULTI,
                                     1,
                                     1),
                          Token::new(String::from("}"), BASE_NONE, RIGHT_CBRACE, 2, 7)];
    let stream = vec!["if", "a", "==", "a", "{", "/*Do something here*/\n", "}"];

    assert_eq!(stream, parse_if(&tok_vector));
}

#[test]
fn test_parse_if_braces_nesting() {
    let tok_vector = vec![Token::new(String::from("if"), BASE_NONE, KEYWORD_IF, 0, 0),
                          Token::new(String::from("("), BASE_NONE, LEFT_BRACKET, 0, 1),
                          Token::new(String::from("a"), BASE_NONE, IDENTIFIER, 0, 2),
                          Token::new(String::from(">"), BASE_BINOP, OP_GT, 0, 3),
                          Token::new(String::from("2"), BASE_VALUE, NUM_INT, 0, 4),
                          Token::new(String::from(")"), BASE_NONE, RIGHT_BRACKET, 0, 5),
                          Token::new(String::from("{"), BASE_NONE, LEFT_CBRACE, 0, 6),
                          Token::new(String::from("/*Do something here*/"),
                                     BASE_COMMENT,
                                     COMMENT_MULTI,
                                     1,
                                     7),
                          Token::new(String::from("if"), BASE_NONE, KEYWORD_IF, 2, 8),
                          Token::new(String::from("("), BASE_NONE, LEFT_BRACKET, 2, 9),
                          Token::new(String::from("a"), BASE_NONE, IDENTIFIER, 2, 10),
                          Token::new(String::from("<"), BASE_BINOP, OP_LT, 2, 11),
                          Token::new(String::from("4"), BASE_VALUE, NUM_INT, 2, 12),
                          Token::new(String::from(")"), BASE_NONE, RIGHT_BRACKET, 2, 13),
                          Token::new(String::from("{"), BASE_NONE, LEFT_CBRACE, 2, 14),
                          Token::new(String::from("b"), BASE_NONE, IDENTIFIER, 3, 15),
                          Token::new(String::from("="), BASE_NONE, OP_ASSIGN, 3, 16),
                          Token::new(String::from("53"), BASE_VALUE, NUM_INT, 3, 17),
                          Token::new(String::from(";"), BASE_VALUE, SEMICOLON, 3, 18),
                          Token::new(String::from("}"), BASE_NONE, RIGHT_CBRACE, 4, 19),
                          Token::new(String::from("b"), BASE_NONE, IDENTIFIER, 5, 20),
                          Token::new(String::from("="), BASE_NONE, OP_ASSIGN, 5, 21),
                          Token::new(String::from("72"), BASE_VALUE, NUM_INT, 5, 22),
                          Token::new(String::from(";"), BASE_NONE, SEMICOLON, 5, 23),
                          Token::new(String::from("}"), BASE_NONE, RIGHT_CBRACE, 6, 24)];
    let stream = vec!["if",
                      "a",
                      ">",
                      "2",
                      "{",
                      "/*Do something here*/\n",
                      "if",
                      "a",
                      "<",
                      "4",
                      "{",
                      "b",
                      "=",
                      "53",
                      ";",
                      "}",
                      "b",
                      "=",
                      "72",
                      ";",
                      "}"];

    assert_eq!(stream, parse_if(&tok_vector));
}

#[test]
fn test_parse_if_no_braces_nesting() {
    let tok_vector = vec![Token::new(String::from("if"), BASE_NONE, KEYWORD_IF, 0, 0),
                          Token::new(String::from("("), BASE_NONE, LEFT_BRACKET, 0, 1),
                          Token::new(String::from("a"), BASE_NONE, IDENTIFIER, 0, 2),
                          Token::new(String::from(">"), BASE_BINOP, OP_GT, 0, 3),
                          Token::new(String::from("2"), BASE_VALUE, NUM_INT, 0, 4),
                          Token::new(String::from(")"), BASE_NONE, RIGHT_BRACKET, 0, 5),
                          Token::new(String::from("if"), BASE_NONE, KEYWORD_IF, 1, 7),
                          Token::new(String::from("("), BASE_NONE, LEFT_BRACKET, 1, 8),
                          Token::new(String::from("a"), BASE_NONE, IDENTIFIER, 1, 9),
                          Token::new(String::from("<"), BASE_BINOP, OP_LT, 1, 10),
                          Token::new(String::from("4"), BASE_VALUE, NUM_INT, 1, 11),
                          Token::new(String::from(")"), BASE_NONE, RIGHT_BRACKET, 1, 12),
                          Token::new(String::from("b"), BASE_NONE, IDENTIFIER, 2, 15),
                          Token::new(String::from("="), BASE_NONE, OP_ASSIGN, 2, 16),
                          Token::new(String::from("53"), BASE_VALUE, NUM_INT, 2, 17),
                          Token::new(String::from(","), BASE_NONE, COMMA, 2, 18),
                          Token::new(String::from("b"), BASE_NONE, IDENTIFIER, 3, 20),
                          Token::new(String::from("="), BASE_NONE, OP_ASSIGN, 3, 21),
                          Token::new(String::from("72"), BASE_VALUE, NUM_INT, 3, 22),
                          Token::new(String::from(";"), BASE_NONE, SEMICOLON, 3, 23)];
    let stream = vec!["if", "a", ">", "2", "{", "if", "a", "<", "4", "{", "b", "=", "53", ";",
                      "b", "=", "72", ";", "}", "}"];

    assert_eq!(stream, parse_if(&tok_vector));
}

#[test]
fn test_parse_if_no_braces() {
    let tok_vector = vec![Token::new(String::from("if"), BASE_NONE, KEYWORD_IF, 0, 0),
                          Token::new(String::from("("), BASE_NONE, LEFT_BRACKET, 0, 1),
                          Token::new(String::from("a"), BASE_NONE, IDENTIFIER, 0, 2),
                          Token::new(String::from("=="), BASE_BINOP, OP_EQU, 0, 3),
                          Token::new(String::from("a"), BASE_NONE, IDENTIFIER, 0, 4),
                          Token::new(String::from(")"), BASE_NONE, RIGHT_BRACKET, 0, 5),
                          Token::new(String::from("a"), BASE_NONE, IDENTIFIER, 1, 6),
                          Token::new(String::from("="), BASE_NONE, OP_ASSIGN, 1, 7),
                          Token::new(String::from("5"), BASE_VALUE, NUM_INT, 1, 8),
                          Token::new(String::from(";"), BASE_NONE, SEMICOLON, 1, 8)];
    let stream = vec!["if", "a", "==", "a", "{", "a", "=", "5", ";", "}"];

    assert_eq!(stream, parse_if(&tok_vector));
}

#[test]
fn test_parse_if_no_braces_inside_braces() {
    let tok_vector = vec![Token::new(String::from("if"), BASE_NONE, KEYWORD_IF, 0, 0),
                          Token::new(String::from("("), BASE_NONE, LEFT_BRACKET, 0, 1),
                          Token::new(String::from("a"), BASE_NONE, IDENTIFIER, 0, 2),
                          Token::new(String::from(">"), BASE_BINOP, OP_GT, 0, 3),
                          Token::new(String::from("2"), BASE_VALUE, NUM_INT, 0, 4),
                          Token::new(String::from(")"), BASE_NONE, RIGHT_BRACKET, 0, 5),
                          Token::new(String::from("{"), BASE_NONE, LEFT_CBRACE, 0, 6),
                          Token::new(String::from("/*Do something here*/"),
                                     BASE_COMMENT,
                                     COMMENT_MULTI,
                                     1,
                                     7),
                          Token::new(String::from("if"), BASE_NONE, KEYWORD_IF, 2, 8),
                          Token::new(String::from("("), BASE_NONE, LEFT_BRACKET, 2, 9),
                          Token::new(String::from("a"), BASE_NONE, IDENTIFIER, 2, 10),
                          Token::new(String::from("<"), BASE_BINOP, OP_LT, 2, 11),
                          Token::new(String::from("4"), BASE_VALUE, NUM_INT, 2, 12),
                          Token::new(String::from(")"), BASE_NONE, RIGHT_BRACKET, 2, 13),
                          Token::new(String::from("b"), BASE_NONE, IDENTIFIER, 3, 15),
                          Token::new(String::from("="), BASE_NONE, OP_ASSIGN, 3, 16),
                          Token::new(String::from("53"), BASE_VALUE, NUM_INT, 3, 17),
                          Token::new(String::from(","), BASE_VALUE, COMMA, 3, 18),
                          Token::new(String::from("b"), BASE_NONE, IDENTIFIER, 5, 20),
                          Token::new(String::from("="), BASE_NONE, OP_ASSIGN, 5, 21),
                          Token::new(String::from("72"), BASE_VALUE, NUM_INT, 5, 22),
                          Token::new(String::from(";"), BASE_NONE, SEMICOLON, 5, 23),
                          Token::new(String::from("a"), BASE_NONE, IDENTIFIER, 5, 24),
                          Token::new(String::from("="), BASE_NONE, OP_ASSIGN, 5, 25),
                          Token::new(String::from("1"), BASE_VALUE, NUM_INT, 5, 26),
                          Token::new(String::from(";"), BASE_NONE, SEMICOLON, 5, 27),
                          Token::new(String::from("}"), BASE_NONE, RIGHT_CBRACE, 6, 28)];
    let stream = vec!["if",
                      "a",
                      ">",
                      "2",
                      "{",
                      "/*Do something here*/\n",
                      "if",
                      "a",
                      "<",
                      "4",
                      "{",
                      "b",
                      "=",
                      "53",
                      ";",
                      "b",
                      "=",
                      "72",
                      ";",
                      "}",
                      "a",
                      "=",
                      "1",
                      ";",
                      "}"];

    assert_eq!(stream, parse_program(&tok_vector));
}

#[test]
fn test_parse_assignment_single() {
    let tok_vector = vec![Token::new(String::from("a"), BASE_NONE, IDENTIFIER, 0, 0),
                          Token::new(String::from("="), BASE_NONE, OP_ASSIGN, 0, 1),
                          Token::new(String::from("5"), BASE_VALUE, NUM_INT, 0, 2),
                          Token::new(String::from(";"), BASE_NONE, SEMICOLON, 0, 3)];
    let stream = vec!["a", "=", "5", ";"];
    assert_eq!(stream, parse_program(&tok_vector));
}

#[test]
fn test_parse_assignment_multiple() {
    let tok_vector = vec![Token::new(String::from("a"), BASE_NONE, IDENTIFIER, 0, 0),
                          Token::new(String::from("="), BASE_NONE, OP_ASSIGN, 0, 1),
                          Token::new(String::from("b"), BASE_NONE, IDENTIFIER, 0, 2),
                          Token::new(String::from("="), BASE_NONE, OP_ASSIGN, 0, 3),
                          Token::new(String::from("c"), BASE_NONE, IDENTIFIER, 0, 4),
                          Token::new(String::from("="), BASE_NONE, OP_ASSIGN, 0, 5),
                          Token::new(String::from("d"), BASE_NONE, IDENTIFIER, 0, 6),
                          Token::new(String::from("="), BASE_NONE, OP_ASSIGN, 0, 7),
                          Token::new(String::from("5"), BASE_VALUE, NUM_INT, 0, 8),
                          Token::new(String::from(";"), BASE_NONE, SEMICOLON, 0, 9)];
    let stream = vec!["d", "=", "5", ";", "c", "=", "d", ";", "b", "=", "c", ";", "a", "=", "b",
                      ";"];
    assert_eq!(stream, parse_program(&tok_vector));
}

#[test]
fn test_parse_assignment_commas() {
    let tok_vector = vec![Token::new(String::from("a"), BASE_NONE, IDENTIFIER, 1, 5),
                          Token::new(String::from("="), BASE_NONE, OP_ASSIGN, 1, 6),
                          Token::new(String::from("1"), BASE_VALUE, NUM_INT, 1, 7),
                          Token::new(String::from(","), BASE_NONE, COMMA, 1, 8),
                          Token::new(String::from("b"), BASE_NONE, IDENTIFIER, 1, 9),
                          Token::new(String::from("="), BASE_NONE, OP_ASSIGN, 1, 10),
                          Token::new(String::from("2"), BASE_VALUE, NUM_INT, 1, 11),
                          Token::new(String::from(","), BASE_NONE, COMMA, 1, 12),
                          Token::new(String::from("c"), BASE_NONE, IDENTIFIER, 1, 13),
                          Token::new(String::from("="), BASE_NONE, OP_ASSIGN, 1, 14),
                          Token::new(String::from("3"), BASE_VALUE, NUM_INT, 1, 15),
                          Token::new(String::from(";"), BASE_NONE, SEMICOLON, 1, 16)];
    let stream = vec!["a", "=", "1", ";", "b", "=", "2", ";", "c", "=", "3", ";"];
    assert_eq!(stream, parse_program(&tok_vector));
}

#[test]
fn test_parse_assignment_binops() {
    let tok_vector = vec![Token::new(String::from("a"), BASE_NONE, IDENTIFIER, 1, 5),
                          Token::new(String::from("="), BASE_NONE, OP_ASSIGN, 1, 6),
                          Token::new(String::from("2"), BASE_VALUE, NUM_INT, 1, 7),
                          Token::new(String::from("+"), BASE_BINOP, OP_PLUS, 1, 8),
                          Token::new(String::from("3"), BASE_VALUE, NUM_INT, 1, 9),
                          Token::new(String::from("-"), BASE_BINOP, OP_MINUS, 1, 10),
                          Token::new(String::from("4"), BASE_VALUE, NUM_INT, 1, 11),
                          Token::new(String::from("/"), BASE_BINOP, OP_DIV, 1, 12),
                          Token::new(String::from("5"), BASE_VALUE, NUM_INT, 1, 13),
                          Token::new(String::from("*"), BASE_BINOP, OP_MUL, 1, 14),
                          Token::new(String::from("6.3"), BASE_VALUE, NUM_FLOAT, 1, 15),
                          Token::new(String::from("%"), BASE_BINOP, OP_MOD, 1, 16),
                          Token::new(String::from("7"), BASE_VALUE, NUM_INT, 1, 17),
                          Token::new(String::from(";"), BASE_NONE, SEMICOLON, 1, 18),];
    let stream = vec!["a", "=", "2", "+", "3", "-", "4", "/", "5", "*", "6.3", "%", "7", ";"];
    assert_eq!(stream, parse_program(&tok_vector));
}

#[test]
fn test_parse_assignment_preops() {
    let mut tok_vector = vec![Token::new(String::from("a"), BASE_NONE, IDENTIFIER, 1, 5),
                              Token::new(String::from("="), BASE_NONE, OP_ASSIGN, 1, 6),
                              Token::new(String::from("++"), BASE_UNOP, OP_INC, 1, 7),
                              Token::new(String::from("b"), BASE_NONE, IDENTIFIER, 1, 8),
                              Token::new(String::from(";"), BASE_NONE, SEMICOLON, 1, 9),];
    let mut stream = vec!["a", "=", "(", "b", "+=1",  ")", ";"];
    assert_eq!(stream, parse_assignment(&tok_vector));

    tok_vector = vec![Token::new(String::from("a"), BASE_NONE, IDENTIFIER, 1, 5),
                      Token::new(String::from("="), BASE_NONE, OP_ASSIGN, 1, 6),
                      Token::new(String::from("--"), BASE_UNOP, OP_DEC, 1, 7),
                      Token::new(String::from("b"), BASE_NONE, IDENTIFIER, 1, 8),
                      Token::new(String::from(";"), BASE_NONE, SEMICOLON, 1, 9),];
    stream = vec!["a", "=", "(", "b", "-=1", ")", ";"];

    assert_eq!(stream, parse_assignment(&tok_vector));
}

#[test]
fn test_parse_assignment_postops() {
    let mut tok_vector = vec![Token::new(String::from("a"), BASE_NONE, IDENTIFIER, 1, 5),
                              Token::new(String::from("="), BASE_NONE, OP_ASSIGN, 1, 6),
                              Token::new(String::from("b"), BASE_NONE, IDENTIFIER, 1, 8),
                              Token::new(String::from("++"), BASE_UNOP, OP_INC, 1, 7),
                              Token::new(String::from(";"), BASE_NONE, SEMICOLON, 1, 9),];
    let mut stream = vec!["a", "=", "b", ";", "b", "+=1", ";"];
    assert_eq!(stream, parse_assignment(&tok_vector));

    tok_vector = vec![Token::new(String::from("a"), BASE_NONE, IDENTIFIER, 1, 5),
                      Token::new(String::from("="), BASE_NONE, OP_ASSIGN, 1, 6),
                      Token::new(String::from("b"), BASE_NONE, IDENTIFIER, 1, 8),
                      Token::new(String::from("--"), BASE_UNOP, OP_DEC, 1, 7),
                      Token::new(String::from(";"), BASE_NONE, SEMICOLON, 1, 9),];
    stream = vec!["a", "=", "b", ";", "b", "-=1", ";"];

    assert_eq!(stream, parse_assignment(&tok_vector));
}

#[test]
fn test_parse_function_no_args() {
    unsafe {
        IN_BLOCK_STMNT = true;
    }
    let tok_vector = vec![Token::new(String::from("int"), BASE_DATATYPE, PRIMITIVE_INT, 0, 0),
                          Token::new(String::from("a"), BASE_NONE, IDENTIFIER, 0, 1),
                          Token::new(String::from("("), BASE_NONE, LEFT_BRACKET, 0, 2),
                          Token::new(String::from(")"), BASE_NONE, RIGHT_BRACKET, 0, 3),
                          Token::new(String::from("{"), BASE_NONE, LEFT_CBRACE, 0, 4),
                          Token::new(String::from("int"), BASE_DATATYPE, PRIMITIVE_INT, 1, 5),
                          Token::new(String::from("a"), BASE_NONE, IDENTIFIER, 1, 6),
                          Token::new(String::from("="), BASE_NONE, OP_ASSIGN, 1, 7),
                          Token::new(String::from("1"), BASE_VALUE, NUM_INT, 1, 8),
                          Token::new(String::from(";"), BASE_NONE, SEMICOLON, 1, 9),
                          Token::new(String::from("}"), BASE_NONE, RIGHT_CBRACE, 2, 10)];
    let stream = vec!["fn", "a", "(", ")", "->", "i32", "{", "let mut", "a", ":", "i32", "=", "1", ";", "}"];
    assert_eq!(stream, parse_program(&tok_vector));
    unsafe {
        IN_BLOCK_STMNT = false;
    }
}

#[test]
fn test_parse_while_braces() {
    let tok_vector = vec![Token::new(String::from("while"), BASE_NONE, KEYWORD_WHILE, 0, 0),
                          Token::new(String::from("("), BASE_NONE, LEFT_BRACKET, 0, 1),
                          Token::new(String::from("a"), BASE_NONE, IDENTIFIER, 0, 2),
                          Token::new(String::from("=="), BASE_BINOP, OP_EQU, 0, 3),
                          Token::new(String::from("a"), BASE_NONE, IDENTIFIER, 0, 4),
                          Token::new(String::from(")"), BASE_NONE, RIGHT_BRACKET, 0, 5),
                          Token::new(String::from("{"), BASE_NONE, LEFT_CBRACE, 0, 6),
                          Token::new(String::from("/*Do something here*/"),
                                     BASE_COMMENT,
                                     COMMENT_MULTI,
                                     1,
                                     1),
                          Token::new(String::from("}"), BASE_NONE, RIGHT_CBRACE, 2, 7)];
    let stream = vec!["while", "a", "==", "a", "{", "/*Do something here*/\n", "}"];

    assert_eq!(stream, parse_while(&tok_vector));
}

#[test]
fn test_parse_while_no_braces() {
    let tok_vector = vec![Token::new(String::from("while"), BASE_NONE, KEYWORD_WHILE, 0, 0),
                          Token::new(String::from("("), BASE_NONE, LEFT_BRACKET, 0, 1),
                          Token::new(String::from("a"), BASE_NONE, IDENTIFIER, 0, 2),
                          Token::new(String::from("=="), BASE_BINOP, OP_EQU, 0, 3),
                          Token::new(String::from("a"), BASE_NONE, IDENTIFIER, 0, 4),
                          Token::new(String::from(")"), BASE_NONE, RIGHT_BRACKET, 0, 5),
                          Token::new(String::from("a"), BASE_NONE, IDENTIFIER, 1, 6),
                          Token::new(String::from("="), BASE_NONE, OP_ASSIGN, 1, 7),
                          Token::new(String::from("5"), BASE_VALUE, NUM_INT, 1, 8),
                          Token::new(String::from(";"), BASE_NONE, SEMICOLON, 1, 8)];
    let stream = vec!["while", "a", "==", "a", "{", "a", "=", "5", ";", "}"];

    assert_eq!(stream, parse_while(&tok_vector));
}

#[test]
fn test_parse_dowhile_braces() {
    let tok_vector = vec![
        Token::new(String::from("do"), BASE_NONE, KEYWORD_DO, 0, 1),
        Token::new(String::from("{"), BASE_NONE, LEFT_CBRACE, 0, 6),
        Token::new(String::from("/*Do something here*/"),
                                     BASE_COMMENT,
                                     COMMENT_MULTI,
                                     1,
                                     1),
        Token::new(String::from("}"), BASE_NONE, RIGHT_CBRACE, 2, 7),
        Token::new(String::from("while"), BASE_NONE, KEYWORD_WHILE, 0, 0),                  
                          Token::new(String::from("("), BASE_NONE, LEFT_BRACKET, 0, 1),
                          Token::new(String::from("a"), BASE_NONE, IDENTIFIER, 0, 2),
                          Token::new(String::from("=="), BASE_BINOP, OP_EQU, 0, 3),
                          Token::new(String::from("a"), BASE_NONE, IDENTIFIER, 0, 4),
                          Token::new(String::from(")"), BASE_NONE, RIGHT_BRACKET, 0, 5),
                          Token::new(String::from(";"), BASE_NONE, SEMICOLON, 0, 8)];
    let stream = vec!["while", "{", "/*Do something here*/\n", "a", "==", "a", "}", "{", "}", ";"];

    assert_eq!(stream, parse_dowhile(&tok_vector));
}

#[test]
fn test_parse_for_braces() {
    let tok_vector = vec![Token::new(String::from("for"), BASE_NONE, KEYWORD_FOR, 0, 0),
                          Token::new(String::from("("), BASE_NONE, LEFT_BRACKET, 0, 1),
                          Token::new(String::from("i"), BASE_NONE, IDENTIFIER, 0, 2),
                          Token::new(String::from("="), BASE_NONE, OP_ASSIGN, 1, 7),
                          Token::new(String::from("0"), BASE_VALUE, NUM_INT, 1, 8),
                          Token::new(String::from(";"), BASE_NONE, SEMICOLON, 1, 8),
                          Token::new(String::from("i"), BASE_NONE, IDENTIFIER, 0, 2),
                          Token::new(String::from("<"), BASE_NONE, OP_LT, 1, 7),
                          Token::new(String::from("23"), BASE_VALUE, NUM_INT, 1, 8),
                          Token::new(String::from(";"), BASE_NONE, SEMICOLON, 1, 8),
                          Token::new(String::from("i"), BASE_NONE, IDENTIFIER, 0, 2),
                          Token::new(String::from("++"), BASE_UNOP, OP_INC, 0, 3),
                          Token::new(String::from(")"), BASE_NONE, RIGHT_BRACKET, 0, 5),
                          Token::new(String::from("{"), BASE_NONE, LEFT_CBRACE, 0, 6),
                          Token::new(String::from("func"), BASE_NONE, IDENTIFIER, 0, 2),
                          Token::new(String::from("("), BASE_NONE, LEFT_BRACKET, 0, 1),
                          Token::new(String::from(")"), BASE_NONE, RIGHT_BRACKET, 0, 5),
                          Token::new(String::from(";"), BASE_NONE, SEMICOLON, 1, 8),
                          Token::new(String::from("}"), BASE_NONE, RIGHT_CBRACE, 2, 7),];
    let stream = vec!["i", "=", "0", ";", "while", "i", "<", "23", "{", "func", "(", ")", ";", "i", "+=1", ";", "}"];

    assert_eq!(stream, parse_for(&tok_vector));
}

#[test]
fn test_parse_for_no_braces() {
    let tok_vector = vec![Token::new(String::from("for"), BASE_NONE, KEYWORD_FOR, 0, 0),
                          Token::new(String::from("("), BASE_NONE, LEFT_BRACKET, 0, 1),
                          Token::new(String::from("i"), BASE_NONE, IDENTIFIER, 0, 2),
                          Token::new(String::from("="), BASE_NONE, OP_ASSIGN, 1, 7),
                          Token::new(String::from("0"), BASE_VALUE, NUM_INT, 1, 8),
                          Token::new(String::from(";"), BASE_NONE, SEMICOLON, 1, 8),
                          Token::new(String::from("i"), BASE_NONE, IDENTIFIER, 0, 2),
                          Token::new(String::from("<"), BASE_NONE, OP_LT, 1, 7),
                          Token::new(String::from("23"), BASE_VALUE, NUM_INT, 1, 8),
                          Token::new(String::from(";"), BASE_NONE, SEMICOLON, 1, 8),
                          Token::new(String::from("i"), BASE_NONE, IDENTIFIER, 0, 2),
                          Token::new(String::from("++"), BASE_UNOP, OP_INC, 0, 3),
                          Token::new(String::from(")"), BASE_NONE, RIGHT_BRACKET, 0, 5),
                        //   Token::new(String::from("{"), BASE_NONE, LEFT_CBRACE, 0, 6),
                          Token::new(String::from("func"), BASE_NONE, IDENTIFIER, 0, 2),
                          Token::new(String::from("("), BASE_NONE, LEFT_BRACKET, 0, 1),
                          Token::new(String::from(")"), BASE_NONE, RIGHT_BRACKET, 0, 5),
                          Token::new(String::from(";"), BASE_NONE, SEMICOLON, 1, 8),];
                        //   Token::new(String::from("}"), BASE_NONE, RIGHT_CBRACE, 2, 7),];
    let stream = vec!["i", "=", "0", ";", "while", "i", "<", "23", "{", "func", "(", ")", ";", "i", "+=1", ";", "}"];

    assert_eq!(stream, parse_for(&tok_vector));
}