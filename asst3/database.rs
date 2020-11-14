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
pub fn handle_request(request: Request, db: & mut Database) 
    -> Response  
{           
    /* Handle a valid request */
    let result = match request.command {
        Command::Insert(values) => 
            handle_insert(db, request.table_id, values),
        Command::Update(id, version, values) => 
             handle_update(db, request.table_id, id, version, values),
        Command::Drop(id) => handle_drop(db, request.table_id, id),
        Command::Get(id) => handle_get(db, request.table_id, id),
        Command::Query(column_id, operator, value) => 
            handle_query(db, request.table_id, column_id, operator, value),
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
 
fn handle_insert(db: & mut Database, table_id: i32, values: Vec<Value>) 
    -> Result<Response, i32> 
{
    //Check if table_id exists in Database
    let mut table_id_exist: bool = false;
    let mut table_object_index: usize = 0;

    for i in 0..db.tables.len() {
        if table_id == db.tables[i].t_id {
            table_id_exist = true;
            table_object_index = i;
        }
    }

    if !table_id_exist {
        return Err(Response::BAD_TABLE);
    }
    
    //Check number of values matches number of columns
    if values.len() != db.tables[table_object_index].t_cols.len() {
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

        if value_type == Value::INTEGER && db.tables[table_object_index].t_cols[i].c_type != Value::INTEGER {
            return Err(Response::BAD_VALUE);
        }
        else if value_type == Value::FLOAT && db.tables[table_object_index].t_cols[i].c_type != Value::FLOAT {
            return Err(Response::BAD_VALUE);
        }
        else if value_type == Value::STRING && db.tables[table_object_index].t_cols[i].c_type != Value::STRING {
            return Err(Response::BAD_VALUE);
        }
        else if value_type == Value::FOREIGN {
            if db.tables[table_object_index].t_cols[i].c_type != Value::FOREIGN {
                return Err(Response::BAD_VALUE);
            }
            else {
                //Check if foreign key reference exists
                //let foreign_table_id = db.tables[table_object_index].t_cols[i].c_ref;
                let mut foreign_key_exist = false;

                for j in 0..db.row_objects.len() {
                    if db.tables[table_object_index].t_cols[i].c_ref == db.row_objects[j].table_id && foreign_value == db.row_objects[j].object_id {
                        foreign_key_exist = true;
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

    //Count number of rows in the table
    for i in 0..db.row_objects.len() {
        if table_id == db.row_objects[i].table_id {
            insert_row_id += 1;
        }
    }

    insert_row_id += 1;
    let version: i64 = 1;
    let response: Response = Response::Insert(insert_row_id, version);

    let new_row: Row = Row::new(table_id, insert_row_id, version, values);
    db.row_objects.push(new_row);

    Ok(response)

}

fn handle_update(db: & mut Database, table_id: i32, object_id: i64, 
    version: i64, values: Vec<Value>) -> Result<Response, i32> 
{
    //Check if table_id exists in Database
    let mut table_id_exist: bool = false;
    let mut table_object_index: usize = 0;

    for i in 0..db.tables.len() {
        if table_id == db.tables[i].t_id {
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

    for i in 0..db.row_objects.len() {
        if table_id == db.row_objects[i].table_id && object_id == db.row_objects[i].object_id {
            object_id_exist = true;
            row_object_index = i;
        }
    }

    if !object_id_exist {
        return Err(Response::NOT_FOUND);
    }

    //Check number of values matches number of columns
    if values.len() != db.tables[table_object_index].t_cols.len() {
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

        if value_type == Value::INTEGER && db.tables[table_object_index].t_cols[i].c_type != Value::INTEGER {
            return Err(Response::BAD_VALUE);
        }
        else if value_type == Value::FLOAT && db.tables[table_object_index].t_cols[i].c_type != Value::FLOAT {
            return Err(Response::BAD_VALUE);
        }
        else if value_type == Value::STRING && db.tables[table_object_index].t_cols[i].c_type != Value::STRING {
            return Err(Response::BAD_VALUE);
        }
        else if value_type == Value::FOREIGN {
            if db.tables[table_object_index].t_cols[i].c_type != Value::FOREIGN {
                return Err(Response::BAD_VALUE);
            }
            else {
                //Check if foreign key reference exists
                //let foreign_table_id = db.tables[table_object_index].t_cols[i].c_ref;
                let mut foreign_key_exist = false;

                for j in 0..db.row_objects.len() {
                    if db.tables[table_object_index].t_cols[i].c_ref == db.row_objects[j].table_id && foreign_value == db.row_objects[j].object_id {
                        foreign_key_exist = true;
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
    
    if db.row_objects[row_object_index].version == version {
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
    let new_version: i64 = db.row_objects[row_object_index].version + 1;
    let response: Response = Response::Update(new_version);

    db.row_objects[row_object_index].version = new_version;
    db.row_objects[row_object_index].values = values;

    Ok(response)

}

fn handle_drop(db: & mut Database, table_id: i32, object_id: i64) 
    -> Result<Response, i32>
{
    Err(Response::UNIMPLEMENTED)
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
    Err(Response::UNIMPLEMENTED)
}
