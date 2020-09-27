import easydb.exception
from .exception import *
import string
import re

def iterable_check(obj):
    try:
        iter(obj)
    except TypeError:
        print("item is not iterable")
        return False

def table_name_check(table_name, existing_names):
    #1.should be string; 
    if isinstance(table_name, str) == False:
        raise TypeError("table name is not a string")
        return False    
    #2.must start with a letter, followed by letters(either case), numbers, or underscore
    if table_name[0].isalpha() == False:
        raise ValueError("table name must start with a letter") 
        return False
    if re.match("^[A-Za-z0-9_]*$", table_name) == None:
        raise ValueError("table name must only contain letters, numbers and underscore")
        return False
    #3.cannot be duplicated  
    for name in existing_names:
        if table_name == name:
            raise ValueError("duplicate table names")
            return False
    
def column_name_check(col_name, existing_names):
    #1.should be string; 
    if isinstance(col_name, str) == False:
        raise TypeError("column name is not a string")
        return False     
    #2.must start with a letter, followed by letters(either case), numbers, or underscore
    if col_name[0].isalpha() == False:
        raise ValueError("column name must start with a letter") 
        return False
    if re.match("^[A-Za-z0-9_]*$", col_name) == None:
        raise ValueError("column name must only contain letters, numbers and underscore")
        return False
    #3.cannot be duplicated  
    for name in existing_names:
        if col_name == name:
            raise ValueError("duplicate column names")
            return False
        
def column_type_check(col_type,table_index,existing_table_names):
    if isinstance(col_type, str):
    #1.cannot reference to current table
        if col_type == existing_table_names[table_index]:
            raise IntegrityError("foreign key causes a cycle: references to current table")
            return False
    #2.cannot reference to nonexistent table  
        if col_type not in existing_table_names:
            raise IntegrityError("foreign key references a nonexistent table")
            return False
    #3.should be one of str, float, int   
    elif col_type != str and col_type != int and col_type != float:
        raise ValueError("column type is not one of strng, float or integer")
        return False