use postgres::{Connection, TlsMode};

pub fn connect(opts: &Opts) -> std::result::Result<Vec<HashMap<String, String>>, DbError> {
    let conn = Connection::connect(opts.connection_string, TlsMode::None)?;
    if let Some(ref sql_text) = opts.sql_text {
        execute_statement(&conn, sql_text)
    } else {
        return Ok(Vec::new());
    }
}

fn execute_statement<'env>(
    conn: &Connection<'env, odbc_safe::AutocommitOn>,
    sql_text: &String,
) -> Result<Vec<HashMap<String, String>>, DbError> {
    let stmt = Statement::with_parent(conn)?;
    let mut results: Vec<HashMap<String, String>> = Vec::new();
    match stmt.exec_direct(&sql_text)? {
        Data(mut stmt) => {
            let col_count = stmt.num_result_cols()? as u16;
            let mut cols: HashMap<u16, odbc::ColumnDescriptor> = HashMap::new();
            for i in 1..(col_count + 1) {
                cols.insert(i, stmt.describe_col(i)?);
            }
            while let Some(mut cursor) = stmt.fetch()? {
                let mut result: HashMap<String, String> = HashMap::new();
                for i in 1..(col_count + 1) {
                    match cursor.get_data::<String>(i as u16)? {
                        Some(val) => {
                            result.insert(cols[&i].name.clone(), val);
                        }
                        None => {
                            result.insert(cols[&i].name.clone(), "".to_string());
                        }
                    }
                }
                results.push(result);
            }
        }
        NoData(_) => println!("Query executed, no data returned"),
    }

    Ok(results)
}