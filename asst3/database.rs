/*
 * database.rs
 *
 * Implementation of EasyDB database internals
 *
 * University of Toronto
 * 2019
 */

use packet::{Command, Request, Response, Value};
use schema::Table;
use std::sync::Arc;
use std::sync::Mutex;
use std::fmt;
 
/* OP codes for the query command */
pub const OP_AL: i32 = 1;
pub const OP_EQ: i32 = 2;
pub const OP_NE: i32 = 3;
pub const OP_LT: i32 = 4;
pub const OP_GT: i32 = 5;
pub const OP_LE: i32 = 6;
pub const OP_GE: i32 = 7;

/* You can implement your Database structure here
 * Q: How you will store your tables into the database? */
pub struct Row {
    pub table_id: i32,
    pub object_id: i64,
    pub version: i64,
    pub values: Vec<Value>
}

impl Row {
    pub fn new(table_identity: i32, object_identity: i64, version_num: i64, value_list: Vec<Value>) -> Row {
        Row {
            table_id: table_identity,
            object_id: object_identity,
            version: version_num,
            values: value_list,
        }
    }
}

impl fmt::Display for Row {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "table, object, version: ({}, {}, {})", self.table_id, self.object_id, self.version)
    }
}

pub struct Database { 
    pub tables: Vec<Table>,
    pub row_objects: Vec<Row>
}

impl Database {
    pub fn new(table_schema: Vec<Table>) -> Database {
        Database {
            tables: table_schema,
            row_objects: vec![],
        }
    }
}

/* Receive the request packet from client and send a response back */
pub fn handle_request(request: Request, db: Vec<Arc<Mutex<Database>>>) 
    -> Response 
{           
    /* Handle a valid request */
    let result = match request.command {
        Command::Insert(values) => {
            if request.table_id <= 0 || request.table_id > db.len() as i32 {
                Err(Response::BAD_TABLE)
            } else {
                
                let mut shared_db = vec![];

                for i in 0..db.len() {
                    shared_db.push(db[i].lock().unwrap());
                }

                let mut shared_db_tables = vec![];

                for i in 0..db.len() {
                    shared_db_tables.push(&mut *(shared_db[i]));
                }

                handle_insert(shared_db_tables, request.table_id, values)
            }
        },
        Command::Update(id, version, values) => {
            if request.table_id <= 0 || request.table_id > db.len() as i32 {
                Err(Response::BAD_TABLE)
            } else {

                let mut shared_db = vec![];

                for i in 0..db.len() {
                    shared_db.push(db[i].lock().unwrap());
                }

                let mut shared_db_tables = vec![];

                for i in 0..db.len() {
                    shared_db_tables.push(&mut *(shared_db[i]));
                }

                handle_update(shared_db_tables, request.table_id, id, version, values)
            }
        },
        Command::Drop(id) => {
            if request.table_id <= 0 || request.table_id > db.len() as i32 {
                 Err(Response::BAD_TABLE)
            } else {

                let mut shared_db = vec![];

                for i in 0..db.len() {
                    shared_db.push(db[i].lock().unwrap());
                }

                let mut shared_db_tables = vec![];

                for i in 0..db.len() {
                    shared_db_tables.push(&mut *(shared_db[i]));
                }

                handle_drop(shared_db_tables, request.table_id, id)
            }
        },
        Command::Get(id) => {
            if request.table_id <= 0 || request.table_id > db.len() as i32 {
                Err(Response::BAD_TABLE) 
            } else {

                let mut shared_db = db[request.table_id as usize - 1].lock().unwrap();
                handle_get(&mut *shared_db, request.table_id, id)
            }
        },
        Command::Query(column_id, operator, value) => {
            if request.table_id <= 0 || request.table_id > db.len() as i32 {
                Err(Response::BAD_TABLE)
            } else {

                let mut shared_db = db[request.table_id as usize - 1].lock().unwrap();
                handle_query(&mut *shared_db, request.table_id, column_id, operator, value)
            }
        },
        /* should never get here */
        Command::Exit => Err(Response::UNIMPLEMENTED),
    };
    
    /* Send back a response */
    match result {
        Ok(response) => response,
        Err(code) => Response::Error(code),
    }
}

/*
 * TODO: Implment these EasyDB functions
 */
 
fn handle_insert(db: Vec<& mut Database>, table_id: i32, values: Vec<Value>) 
    -> Result<Response, i32> 
{
    //db index
    let db_index = table_id as usize - 1;

    //Check if table_id exists in Database
    let mut table_id_exist: bool = false;
    let mut table_object_index: usize = 0;

    for i in 0..db[db_index].tables.len() {
        if table_id == db[db_index].tables[i].t_id {
            table_id_exist = true;
            table_object_index = i;
        }
    }

    if !table_id_exist {
        return Err(Response::BAD_TABLE);
    }
    
    //Check number of values matches number of columns
    if values.len() != db[db_index].tables[table_object_index].t_cols.len() {
        return Err(Response::BAD_ROW);
    }

    //Check for column type mismatches and bad foreign key
    for i in 0..values.len() {
        let value_type: i32;
        let mut foreign_value: i64 = 0;

        //Find value's type
        match &values[i] {
            Value::Null => value_type = Value::NULL,
            Value::Integer(val) => value_type = Value::INTEGER,
            Value::Float(val) => value_type = Value::FLOAT,
            Value::Text(val) => value_type = Value::STRING,
            Value::Foreign(val) => {
                value_type = Value::FOREIGN;
                foreign_value = *val;
            },
        }

        if value_type == Value::INTEGER && db[db_index].tables[table_object_index].t_cols[i].c_type != Value::INTEGER {
            return Err(Response::BAD_VALUE);
        }
        else if value_type == Value::FLOAT && db[db_index].tables[table_object_index].t_cols[i].c_type != Value::FLOAT {
            return Err(Response::BAD_VALUE);
        }
        else if value_type == Value::STRING && db[db_index].tables[table_object_index].t_cols[i].c_type != Value::STRING {
            return Err(Response::BAD_VALUE);
        }
        else if value_type == Value::FOREIGN {
            if db[db_index].tables[table_object_index].t_cols[i].c_type != Value::FOREIGN {
                return Err(Response::BAD_VALUE);
            }
            else {
                //Check if foreign key reference exists
                //let foreign_table_id = db[db_index].tables[table_object_index].t_cols[i].c_ref;
                let mut foreign_key_exist = false;

                for j in 0..db.len() {
                    for k in 0..db[j].row_objects.len() {
                        if db[db_index].tables[table_object_index].t_cols[i].c_ref == db[j].row_objects[k].table_id && foreign_value == db[j].row_objects[k].object_id {
                            foreign_key_exist = true;
                        }
                    }
                }

                if foreign_value == 0 {
                    foreign_key_exist = true;
                }

                if !foreign_key_exist {
                    return Err(Response::BAD_FOREIGN);
                }
            }
        }
    }


    //All checks passed
    //Insert the row
    let mut insert_row_id: i64 = 0;

    //Set object_id to be last row's object_id + 1
    for i in 0..db[db_index].row_objects.len() {
        if table_id == db[db_index].row_objects[i].table_id {
            insert_row_id = db[db_index].row_objects[i].object_id;
        }
    }

    insert_row_id += 1;
    let version: i64 = 1;
    let response: Response = Response::Insert(insert_row_id, version);

    let new_row: Row = Row::new(table_id, insert_row_id, version, values);
    db[db_index].row_objects.push(new_row);
   
    Ok(response)
}

fn handle_update(db: Vec<& mut Database>, table_id: i32, object_id: i64, 
    version: i64, values: Vec<Value>) -> Result<Response, i32> 
{
    //db index
    let db_index = table_id as usize - 1;

    //Check if table_id exists in Database
    let mut table_id_exist: bool = false;
    let mut table_object_index: usize = 0;

    for i in 0..db[db_index].tables.len() {
        if table_id == db[db_index].tables[i].t_id {
            table_id_exist = true;
            table_object_index = i;
        }
    }

    if !table_id_exist {
        return Err(Response::BAD_TABLE);
    }
    
    //Check if object_id exists in the table
    let mut object_id_exist: bool = false;
    let mut row_object_index: usize = 0;

    for i in 0..db[db_index].row_objects.len() {
        if table_id == db[db_index].row_objects[i].table_id && object_id == db[db_index].row_objects[i].object_id {
            object_id_exist = true;
            row_object_index = i;
        }
    }

    if !object_id_exist {
        return Err(Response::NOT_FOUND);
    }

    //Check number of values matches number of columns
    if values.len() != db[db_index].tables[table_object_index].t_cols.len() {
        return Err(Response::BAD_ROW);
    }

    //Check for column type mismatches and bad foreign key
    for i in 0..values.len() {
        let value_type: i32;
        let mut foreign_value: i64 = 0;

        //Find value's type
        match &values[i] {
            Value::Null => value_type = Value::NULL,
            Value::Integer(val) => value_type = Value::INTEGER,
            Value::Float(val) => value_type = Value::FLOAT,
            Value::Text(val) => value_type = Value::STRING,
            Value::Foreign(val) => {
                value_type = Value::FOREIGN;
                foreign_value = *val;
            },
        }

        if value_type == Value::INTEGER && db[db_index].tables[table_object_index].t_cols[i].c_type != Value::INTEGER {
            return Err(Response::BAD_VALUE);
        }
        else if value_type == Value::FLOAT && db[db_index].tables[table_object_index].t_cols[i].c_type != Value::FLOAT {
            return Err(Response::BAD_VALUE);
        }
        else if value_type == Value::STRING && db[db_index].tables[table_object_index].t_cols[i].c_type != Value::STRING {
            return Err(Response::BAD_VALUE);
        }
        else if value_type == Value::FOREIGN {
            if db[db_index].tables[table_object_index].t_cols[i].c_type != Value::FOREIGN {
                return Err(Response::BAD_VALUE);
            }
            else {
                //Check if foreign key reference exists
                //let foreign_table_id = db[db_index].tables[table_object_index].t_cols[i].c_ref;
                let mut foreign_key_exist = false;

                for j in 0.. db.len() {
                    for k in 0..db[j].row_objects.len() {
                        if db[db_index].tables[table_object_index].t_cols[i].c_ref == db[j].row_objects[k].table_id && foreign_value == db[j].row_objects[k].object_id {
                            foreign_key_exist = true;
                        }
                    }
                }

                if foreign_value == 0 {
                    foreign_key_exist = true;
                }

                if !foreign_key_exist {
                    return Err(Response::BAD_FOREIGN);
                }

            }
        }

    }

    //Check if version number matches or if version = 0
    let mut version_match: bool = false;
    
    if db[db_index].row_objects[row_object_index].version == version {
        version_match = true;
    }
    else if version == 0 {
        version_match = true;
    }
    
    if !version_match {
        return Err(Response::TXN_ABORT);
    }

    //All checks passed
    //Update the row
    let new_version: i64 = db[db_index].row_objects[row_object_index].version + 1;
    let response: Response = Response::Update(new_version);

    db[db_index].row_objects[row_object_index].version = new_version;
    db[db_index].row_objects[row_object_index].values = values;

    Ok(response)

}

fn handle_drop(db: Vec<& mut Database>, table_id: i32, object_id: i64) 
    -> Result<Response, i32>
{

    
    let mut table_index: usize = 0;
    let mut table_id_exist: bool = false;
    let mut schema_has_foreign: bool = false;
    
    for i in 0..db[0].tables.len() {
        //Check if table_id exists in Database
        if db[0].tables[i].t_id == table_id {
            table_id_exist = true;
            table_index = i;
        }
        
        //check if the schema has any foreign
        for j in 0..db[0].tables[i].t_cols.len() {
            if db[0].tables[i].t_cols[j].c_type == Value::FOREIGN {
                schema_has_foreign = true;
            }
        }
    }
    
    if !table_id_exist {
        return Err(Response::BAD_TABLE);
    }
    
    
    let mut object_id_exist: bool = false;
    let mut row_object_index: usize = 0;

    for i in 0..db[table_index].row_objects.len() {
        if object_id == db[table_index].row_objects[i].object_id {
            object_id_exist = true;
            row_object_index = i;
        }
    }

    //Check if object_id exists in the table
    if !object_id_exist {
        return Err(Response::NOT_FOUND);
    }
    
    //only when schema has foreign, find foreigners
    let mut ref_object = Vec::new();
    
    if schema_has_foreign {
        let first_ref_object = find_referenced_row(db, table_index, row_object_index);
        
        if first_ref_object.len() != 0 {
            for i in 0..first_ref_object.len() {
                //push the first foreigners
                ref_object.push(first_ref_object[i]);
                
                //find if there is any secondary foreigners
                let second_ref_object = find_referenced_row(db, first_ref_object[i].0 + 1, first_ref_object[i].1 + 1);
                if second_ref_object.len() != 0 {
                    for j in 0..second_ref_object.len() {
                        ref_object.push(second_ref_object[j]);
                    }
                }
            }
        }
    }
    
    //start dropping
    db[table_index].row_objects.remove(row_object_index);
    
    if ref_object.len() != 0 {
        ref_object.sort();
        ref_object.dedup();
        
        let mut removal_count: usize = 1;
        
        for i in 0..ref_object.len() {
            db[ref_object[i].0].row_objects.remove(ref_object[i].1 - removal_count);
            removal_count += 1;
        }
    }
    
    Ok(Response::Drop)
}

fn handle_get(db: & Database, table_id: i32, object_id: i64) 
    -> Result<Response, i32>
{
    //Check if table_id exists in Database
    let mut table_id_exist: bool = false;

    for i in 0..db.tables.len() {
        if table_id == db.tables[i].t_id {
            table_id_exist = true;
        }
    }

    if !table_id_exist {
        return Err(Response::BAD_TABLE);
    }
    
    //Check if object_id exists in the table
    let mut object_id_exist: bool = false;
    let mut row_object_index: usize = 0;

    for i in 0..db.row_objects.len() {
        if table_id == db.row_objects[i].table_id && object_id == db.row_objects[i].object_id {
            object_id_exist = true;
            row_object_index = i;
        }
    }

    if !object_id_exist {
        return Err(Response::NOT_FOUND);
    }

    //All checks pass
    //Get row from table
    let version: i64 = db.row_objects[row_object_index].version;
    
    Ok(Response::Get(version, &db.row_objects[row_object_index].values))
}

fn handle_query(db: & Database, table_id: i32, column_id: i32,
    operator: i32, other: Value) 
    -> Result<Response, i32>
{
    let mut matched_results = Vec::new();

    //Check if table_id exists in Database
    let mut table_id_exist: bool = false;

    for i in 0..db.tables.len() {
        if table_id == db.tables[i].t_id {
            table_id_exist = true;
        }
    }

    if !table_id_exist {
        return Err(Response::BAD_TABLE);
    }
    
    //column infomation
    let mut col_id_exist: bool = false;
    let mut col_index: usize = 0;
    let mut col_type: i32 = 0;
    
    //column_id must be zero for OP_AL
    if operator == OP_AL && column_id != 0 {
        return Err(Response::BAD_QUERY);
    }
    
    for i in 0..db.tables.len() {
        for j in 0..db.tables[i].t_cols.len() {
            
            if table_id == db.tables[i].t_id && column_id == db.tables[i].t_cols[j].c_id {
                col_id_exist = true;
                col_index = j;
                col_type = db.tables[i].t_cols[j].c_type;
                
                //only EQ and NE are supported for foreign and id
                if col_type == Value::FOREIGN || db.tables[i].t_cols[j].c_name == "id" {
                    if operator != OP_EQ && operator != OP_NE && operator != OP_AL{
                        return Err(Response::BAD_QUERY);
                    }
                }
            }
        }
    }
    
    //Invalid column_id
    if !col_id_exist {
        if operator != OP_AL {
            return Err(Response::BAD_QUERY); 
        }
    }
    
    //case OP_AL: regard less column_id and other
    if operator == OP_AL {
        for i in 0..db.row_objects.len() {
            if table_id == db.row_objects[i].table_id {
                matched_results.push(db.row_objects[i].object_id);
            }
        }
    }
    
    //Parse other type and value
    let other_type: i32;
    let mut other_val_int: i64 = 0;
    let mut other_val_float: f64 = 0.0;
    let mut other_val_text: String = String::from(' ');
    let mut other_val_foreign: i64 = 0;
    
    match &other {
        Value::Null => other_type = Value::NULL,
        Value::Integer(val) => {
            other_type = Value::INTEGER;
            other_val_int = *val;
        },
        Value::Float(val) => {
            other_type = Value::FLOAT;
            other_val_float = *val;
        },
        Value::Text(val) => {
            other_type = Value::STRING;
            other_val_text = val.to_string();
        },
        Value::Foreign(val) => {
            other_type = Value::FOREIGN;
            other_val_foreign = *val;
        },
    }
    
    //Invalid value type
    if col_type != other_type {
        return Err(Response::BAD_QUERY); 
    }

    let mut iter: Value;

    for i in 0..db.row_objects.len() {
        if table_id == db.row_objects[i].table_id {
            
            if operator != OP_AL {
                let mut iter_val_int: i64 = 0;
                let mut iter_val_float: f64 = 0.0;
                let mut iter_val_text: String = String::from(' ');
                let mut iter_val_foreign: i64 = 0;
                
                match &db.row_objects[i].values[col_index] {
                    Value::Null => (),
                    Value::Integer(val) => iter_val_int = *val,
                    Value::Float(val) => iter_val_float = *val,
                    Value::Text(val) => iter_val_text = val.to_string(),
                    Value::Foreign(val) => iter_val_foreign = *val,
                }
                
                //case FOREIGN
                if other_type == Value::FOREIGN {
                    if iter_val_foreign == other_val_foreign && operator == OP_EQ {
                        matched_results.push(db.row_objects[i].object_id);
                    }
                    else if iter_val_foreign != other_val_foreign && operator == OP_NE {
                        matched_results.push(db.row_objects[i].object_id);
                    }
                }
                
                //case INTEGER 
                else if other_type == Value::INTEGER {
                    if iter_val_int == other_val_int {
                        if operator == OP_EQ || operator == OP_LE || operator == OP_GE {
                            matched_results.push(db.row_objects[i].object_id);
                        }
                    }
                    else if iter_val_int < other_val_int {
                        if operator == OP_LT || operator == OP_LE || operator == OP_NE {
                            matched_results.push(db.row_objects[i].object_id);
                        }
                    }
                    else if iter_val_int > other_val_int {
                        if operator == OP_GT || operator == OP_GE || operator == OP_NE {
                            matched_results.push(db.row_objects[i].object_id);
                        }
                    }
                }
                
                //case FLOAT
                else if other_type == Value::FLOAT{
                    if iter_val_float == other_val_float {
                        if operator == OP_EQ || operator == OP_LE || operator == OP_GE {
                            matched_results.push(db.row_objects[i].object_id);
                        }
                    }
                    else if iter_val_float < other_val_float {
                        if operator == OP_LT || operator == OP_LE || operator == OP_NE {
                            matched_results.push(db.row_objects[i].object_id);
                        }
                    }
                    else if iter_val_float > other_val_float {
                        if operator == OP_GT || operator == OP_GE || operator == OP_NE {
                            matched_results.push(db.row_objects[i].object_id);
                        }
                    }
                }
                
                //case STRING
                else if other_type == Value::STRING{
                    if iter_val_text == other_val_text {
                        if operator == OP_EQ || operator == OP_LE || operator == OP_GE {
                            matched_results.push(db.row_objects[i].object_id);
                        }
                    }
                    else if iter_val_text < other_val_text {
                        if operator == OP_LT || operator == OP_LE || operator == OP_NE {
                            matched_results.push(db.row_objects[i].object_id);
                        }
                    }
                    else if iter_val_text > other_val_text {
                        if operator == OP_GT || operator == OP_GE || operator == OP_NE {
                            matched_results.push(db.row_objects[i].object_id);
                        }
                    }
                }
            }
        }
    }

    let response: Response = Response::Query(matched_results);
    Ok(response)
}


//find all rows which reference to the given row
fn find_referenced_row(db: Vec<& mut Database>, table_index: usize, object_index: usize) 
    -> Vec<(usize, usize)>
{
    let mut results = Vec::new();
    let mut ref_tid_cid = Vec::new();
    
    //save table id and object id
    let table_id = db[table_index].row_objects[object_index].table_id;
    let object_id = db[table_index].row_objects[object_index].object_id;
    
    
    //loop through schema (tables) to see if there is any column referencing to the given row's table
    //push to ref_tid_cid as (table id, column index)
    for i in 0..db[0].tables.len(){
        for j in 0..db[0].tables[i].t_cols.len() {
            if db[0].tables[i].t_cols[j].c_type == Value::FOREIGN 
            && db[0].tables[i].t_cols[j].c_ref == table_id {
                ref_tid_cid.push((db[0].tables[i].t_id, j));
            }
        }
    }
    
    //loop through row_objects, check for (table id, column index) in ref_tid_cid
    //if the value of the field is referencing to the given object
    //push (db index, row_objects index) to results  
    
    for k in 0..ref_tid_cid.len() {
        
        //table id: ref_tid_cid[k].0
        //column index: ref_tid_cid[k].1
        
        for m in 0..db.len(){
            for i in 0..db[m].row_objects.len() {
                if db[m].row_objects[i].table_id == ref_tid_cid[k].0 {
                    
                    //get the foreign value of this field
                    let mut field_foreign_value: i64 = 0;  
                    match &db[m].row_objects[i].values[ref_tid_cid[k].1] {
                        Value::Foreign(val) => field_foreign_value = *val,
                        _ => (),
                    }
                    
                    //check if the foreign value matches object_id
                    if field_foreign_value == object_id {
                        results.push((m,i));
                    }
                }
            }
        }
        
    }
    
    return results;
}

