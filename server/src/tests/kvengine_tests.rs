/*
 * Created on Thu Sep 10 2020
 *
 * This file is a part of TerrabaseDB
 * Copyright (c) 2020, Sayan Nandan <ohsayan at outlook dot com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 *
*/

#![allow(unused_variables)]
use super::{fresp, start_server, terrapipe, TcpStream};
use crate::__func__;
use crate::tests::ADDR;
use tokio::prelude::*;

#[tokio::test]
/// Test a HEYA query: The server should return HEY!
async fn test_heya() {
    let server = start_server(None).await;
    let mut stream = TcpStream::connect(ADDR).await.unwrap();
    let heya = terrapipe::proc_query("HEYA");
    stream.write_all(&heya).await.unwrap();
    let res_should_be = "#2\n*1\n#2\n&1\n+4\nHEY!\n".as_bytes().to_owned();
    let mut response = vec![0; res_should_be.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(response, res_should_be, "{}", __func__!());
}

#[tokio::test]
/// Test a GET query: for a non-existing key
async fn test_get_single_nil() {
    let server = start_server(None).await;
    let mut stream = TcpStream::connect(ADDR).await.unwrap();
    let get_single_nil = terrapipe::proc_query("GET x");
    stream.write_all(&get_single_nil).await.unwrap();
    let mut response = vec![0; fresp::R_NIL.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(response, fresp::R_NIL.to_owned(), "{}", __func__!());
}

#[tokio::test]
/// Test a GET query: for an existing key
async fn test_get_single_okay() {
    let server = start_server(None).await;
    let mut stream = TcpStream::connect(ADDR).await.unwrap();
    let mut stream = set_values("x 100", 1, stream).await;
    let get_single_nil = terrapipe::proc_query("GET x");
    stream.write_all(&get_single_nil).await.unwrap();
    let res_should_be = "#2\n*1\n#2\n&1\n+3\n100\n".as_bytes().to_owned();
    let mut response = vec![0; res_should_be.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(response, res_should_be, "{}", __func__!());
}

#[tokio::test]
/// Test a GET query with an incorrect number of arguments
async fn test_get_syntax_error() {
    let server = start_server(None).await;
    let mut stream = TcpStream::connect(ADDR).await.unwrap();
    let syntax_error = terrapipe::proc_query("GET");
    stream.write_all(&syntax_error).await.unwrap();
    let mut response = vec![0; fresp::R_ACTION_ERR.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(
        response,
        fresp::R_ACTION_ERR.to_owned(),
        "{}: With zero arg(s)",
        __func__!()
    );
    let syntax_error = terrapipe::proc_query("GET one two");
    stream.write_all(&syntax_error).await.unwrap();
    let mut response = vec![0; fresp::R_ACTION_ERR.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(
        response,
        fresp::R_ACTION_ERR.to_owned(),
        "{}: With two arg(s)",
        __func__!()
    );
}

/// Set a couple of values, which are to be passed as a list of whitespace separated values
///
///
/// `howmany` is the number of values, which really depends on the calling query:
/// it can be n/2 for set, n/1 for get and so on. To avoid unnecessary complexity,
/// we'll tell the caller to explicitly specify how many keys we should expect
async fn set_values<T>(
    values_split_with_whitespace: T,
    homwany: usize,
    mut stream: TcpStream,
) -> TcpStream
where
    T: AsRef<str>,
{
    let mut query = String::from("MSET ");
    query.push_str(values_split_with_whitespace.as_ref());
    let count_bytes_len = homwany.to_string().as_bytes().len();
    let q = terrapipe::proc_query(query);
    stream.write_all(&q).await.unwrap();
    let res_should_be = format!("#2\n*1\n#2\n&1\n:{}\n{}\n", count_bytes_len, homwany).into_bytes();
    let mut response = vec![0; res_should_be.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(response, res_should_be, "{}", __func__!());
    stream
}

#[tokio::test]
/// Test a SET query: SET a non-existing key, which should return code: 0
async fn test_set_single_okay() {
    let server = start_server(None).await;
    let mut stream = TcpStream::connect(ADDR).await.unwrap();
    let set_single_okay = terrapipe::proc_query("SET x 100");
    stream.write_all(&set_single_okay).await.unwrap();
    let mut response = vec![0; fresp::R_OKAY.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(response, fresp::R_OKAY.to_owned(), "{}", __func__!());
}

#[tokio::test]
/// Test a SET query: SET an existing key, which should return code: 2
async fn test_set_single_overwrite_error() {
    let server = start_server(None).await;
    let mut stream = TcpStream::connect(ADDR).await.unwrap();
    let mut stream = set_values("x 100", 1, stream).await;
    let set_single_code_2 = terrapipe::proc_query("SET x 200");
    stream.write_all(&set_single_code_2).await.unwrap();
    let mut response = vec![0; fresp::R_OVERWRITE_ERR.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(
        response,
        fresp::R_OVERWRITE_ERR.to_owned(),
        "{}",
        __func__!()
    );
}

#[tokio::test]
/// Test a SET query with incorrect number of arugments
async fn test_set_syntax_error() {
    let server = start_server(None).await;
    let mut stream = TcpStream::connect(ADDR).await.unwrap();
    let syntax_error = terrapipe::proc_query("SET");
    stream.write_all(&syntax_error).await.unwrap();
    let mut response = vec![0; fresp::R_ACTION_ERR.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(
        response,
        fresp::R_ACTION_ERR.to_owned(),
        "{}: With zero arg(s)",
        __func__!()
    );
    let syntax_error = terrapipe::proc_query("SET one");
    stream.write_all(&syntax_error).await.unwrap();
    let mut response = vec![0; fresp::R_ACTION_ERR.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(
        response,
        fresp::R_ACTION_ERR.to_owned(),
        "{}: With one arg(s)",
        __func__!()
    );
    let syntax_error = terrapipe::proc_query("SET one 1 two 2");
    stream.write_all(&syntax_error).await.unwrap();
    let mut response = vec![0; fresp::R_ACTION_ERR.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(
        response,
        fresp::R_ACTION_ERR.to_owned(),
        "{}: With four arg(s)",
        __func__!()
    );
}

#[tokio::test]
/// Test an UPDATE query: which should return code: 0
async fn test_update_single_okay() {
    let server = start_server(None).await;
    let mut stream = TcpStream::connect(ADDR).await.unwrap();
    let mut stream = set_values("x 100", 1, stream).await;
    let update_single_okay = terrapipe::proc_query("UPDATE x 200");
    stream.write_all(&update_single_okay).await.unwrap();
    let mut response = vec![0; fresp::R_OKAY.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(response, fresp::R_OKAY.to_owned(), "{}", __func__!());
}

#[tokio::test]
/// Test an UPDATE query: which should return code: 1
async fn test_update_single_nil() {
    let server = start_server(None).await;
    let mut stream = TcpStream::connect(ADDR).await.unwrap();
    let update_single_okay = terrapipe::proc_query("UPDATE x 200");
    stream.write_all(&update_single_okay).await.unwrap();
    let mut response = vec![0; fresp::R_NIL.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(response, fresp::R_NIL.to_owned(), "{}", __func__!());
}

#[tokio::test]
async fn test_update_syntax_error() {
    let server = start_server(None).await;
    let mut stream = TcpStream::connect(ADDR).await.unwrap();
    let syntax_error = terrapipe::proc_query("UPDATE");
    stream.write_all(&syntax_error).await.unwrap();
    let mut response = vec![0; fresp::R_ACTION_ERR.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(
        response,
        fresp::R_ACTION_ERR.to_owned(),
        "{}: With zero arg(s)",
        __func__!()
    );
    let syntax_error = terrapipe::proc_query("UPDATE one");
    stream.write_all(&syntax_error).await.unwrap();
    let mut response = vec![0; fresp::R_ACTION_ERR.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(
        response,
        fresp::R_ACTION_ERR.to_owned(),
        "{}: With one arg(s)",
        __func__!()
    );
    let syntax_error = terrapipe::proc_query("UPDATE one 1 two 2");
    stream.write_all(&syntax_error).await.unwrap();
    let mut response = vec![0; fresp::R_ACTION_ERR.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(
        response,
        fresp::R_ACTION_ERR.to_owned(),
        "{}: With four arg(s)",
        __func__!()
    );
}

#[tokio::test]
/// Test a DEL query: which should return int 0
async fn test_del_single_zero() {
    let server = start_server(None).await;
    let mut stream = TcpStream::connect(ADDR).await.unwrap();
    let update_single_okay = terrapipe::proc_query("DEL x");
    stream.write_all(&update_single_okay).await.unwrap();
    let res_should_be = "#2\n*1\n#2\n&1\n:1\n0\n".as_bytes().to_owned();
    let mut response = vec![0; res_should_be.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(response, res_should_be, "{}", __func__!());
}

#[tokio::test]
/// Test a DEL query: which should return int 1
async fn test_del_single_one() {
    let server = start_server(None).await;
    let mut stream = TcpStream::connect(ADDR).await.unwrap();
    let mut stream = set_values("x 100", 1, stream).await;
    let update_single_okay = terrapipe::proc_query("DEL x");
    stream.write_all(&update_single_okay).await.unwrap();
    let res_should_be = "#2\n*1\n#2\n&1\n:1\n1\n".as_bytes().to_owned();
    let mut response = vec![0; res_should_be.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(response, res_should_be, "{}", __func__!());
}

#[tokio::test]
/// Test a DEL query: which should return the number of keys deleted
async fn test_del_multiple() {
    let server = start_server(None).await;
    let mut stream = TcpStream::connect(ADDR).await.unwrap();
    let mut stream = set_values("x 100 y 200 z 300", 3, stream).await;
    let update_single_okay = terrapipe::proc_query("DEL x y z");
    stream.write_all(&update_single_okay).await.unwrap();
    let res_should_be = "#2\n*1\n#2\n&1\n:1\n3\n".as_bytes().to_owned();
    let mut response = vec![0; res_should_be.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(response, res_should_be, "{}", __func__!());
}

#[tokio::test]
/// Test a DEL query with an incorrect number of arguments
async fn test_del_syntax_error() {
    let server = start_server(None).await;
    let mut stream = TcpStream::connect(ADDR).await.unwrap();
    let syntax_error = terrapipe::proc_query("DEL");
    stream.write_all(&syntax_error).await.unwrap();
    let mut response = vec![0; fresp::R_ACTION_ERR.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(response, fresp::R_ACTION_ERR.to_owned(), "{}", __func__!());
}

#[tokio::test]
/// Test an EXISTS query for mixed outcomes
async fn test_exists_multiple_mixed() {
    let server = start_server(None).await;
    let mut stream = TcpStream::connect(ADDR).await.unwrap();
    let mut stream = set_values("x ex y why z zed", 3, stream).await;
    let query = terrapipe::proc_query("EXISTS x");
    stream.write_all(&query).await.unwrap();
    let res_should_be = "#2\n*1\n#2\n&1\n:1\n1\n".to_owned().into_bytes();
    let mut response = vec![0; res_should_be.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(response, res_should_be, "{}: With one arg(s)", __func__!());
    let query = terrapipe::proc_query("EXISTS x y z");
    stream.write_all(&query).await.unwrap();
    let res_should_be = "#2\n*1\n#2\n&1\n:1\n3\n".to_owned().into_bytes();
    let mut response = vec![0; res_should_be.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(
        response,
        res_should_be,
        "{}: With three arg(s)",
        __func__!()
    );
}

#[tokio::test]
/// Test an EXISTS query with an incorrect number of arguments
async fn test_exists_syntax_error() {
    let server = start_server(None).await;
    let mut stream = TcpStream::connect(ADDR).await.unwrap();
    let syntax_error = terrapipe::proc_query("EXISTS");
    stream.write_all(&syntax_error).await.unwrap();
    let mut response = vec![0; fresp::R_ACTION_ERR.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(response, fresp::R_ACTION_ERR.to_owned(), "{}", __func__!());
}

#[tokio::test]
/// Test an MGET query on a single existing key
async fn test_mget_single_okay() {
    let server = start_server(None).await;
    let mut stream = TcpStream::connect(ADDR).await.unwrap();
    let mut stream = set_values("x 100", 1, stream).await;
    let query = terrapipe::proc_query("MGET x");
    stream.write_all(&query).await.unwrap();
    let res_should_be = "#2\n*1\n#2\n&1\n+3\n100\n".to_owned().into_bytes();
    let mut response = vec![0; res_should_be.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(response, res_should_be, "{}", __func__!());
}

#[tokio::test]
/// Test an MGET query on multiple existing keys
async fn test_mget_multiple_allokay() {
    let server = start_server(None).await;
    let mut stream = TcpStream::connect(ADDR).await.unwrap();
    let mut stream = set_values("x 100 y 200 z 300", 3, stream).await;
    let query = terrapipe::proc_query("MGET x y z");
    stream.write_all(&query).await.unwrap();
    let res_should_be = "#2\n*1\n#2\n&3\n+3\n100\n+3\n200\n+3\n300\n"
        .to_owned()
        .into_bytes();
    let mut response = vec![0; res_should_be.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(response, res_should_be, "{}", __func__!());
}

#[tokio::test]
/// Test an MGET query with different outcomes
async fn test_mget_multiple_mixed() {
    let server = start_server(None).await;
    let mut stream = TcpStream::connect(ADDR).await.unwrap();
    let mut stream = set_values("x 100 z 200", 2, stream).await;
    let query = terrapipe::proc_query("mget x y z");
    stream.write_all(&query).await.unwrap();
    let res_should_be = "#2\n*1\n#2\n&3\n+3\n100\n!1\n1\n+3\n200\n"
        .to_owned()
        .into_bytes();
    let mut response = vec![0; res_should_be.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(response, res_should_be, "{}", __func__!());
}

#[tokio::test]
/// Test an MGET query with an incorrect number of arguments
async fn test_mget_syntax_error() {
    let server = start_server(None).await;
    let mut stream = TcpStream::connect(ADDR).await.unwrap();
    let syntax_error = terrapipe::proc_query("MGET");
    stream.write_all(&syntax_error).await.unwrap();
    let mut response = vec![0; fresp::R_ACTION_ERR.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(response, fresp::R_ACTION_ERR.to_owned(), "{}", __func__!());
}

#[tokio::test]
/// Test an MSET query with a single non-existing keys
async fn test_mset_single_okay() {
    let server = start_server(None).await;
    let mut stream = TcpStream::connect(ADDR).await.unwrap();
    let query = terrapipe::proc_query("MSET x ex");
    stream.write_all(&query).await.unwrap();
    let res_should_be = "#2\n*1\n#2\n&1\n:1\n1\n".to_owned().into_bytes();
    let mut response = vec![0; res_should_be.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(response, res_should_be, "{}", __func__!());
}

#[tokio::test]
/// Test an MSET query with non-existing keys
async fn test_mset_multiple_okay() {
    let server = start_server(None).await;
    let mut stream = TcpStream::connect(ADDR).await.unwrap();
    let query = terrapipe::proc_query("MSET x ex y why z zed");
    stream.write_all(&query).await.unwrap();
    let res_should_be = "#2\n*1\n#2\n&1\n:1\n3\n".to_owned().into_bytes();
    let mut response = vec![0; res_should_be.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(response, res_should_be, "{}", __func__!());
}

#[tokio::test]
/// Test an MSET query with a mixed set of outcomes
async fn test_mset_multiple_mixed() {
    let server = start_server(None).await;
    let mut stream = TcpStream::connect(ADDR).await.unwrap();
    let mut stream = set_values("x ex", 1, stream).await;
    let query = terrapipe::proc_query("MSET x ex y why z zed");
    stream.write_all(&query).await.unwrap();
    let res_should_be = "#2\n*1\n#2\n&1\n:1\n2\n".to_owned().into_bytes();
    let mut response = vec![0; res_should_be.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(
        response,
        res_should_be,
        "{}: With 3 k/v pair(s)",
        __func__!()
    );
    // Now all the keys have been set, so we should get a 0
    let query = terrapipe::proc_query("MSET x ex y why z zed");
    stream.write_all(&query).await.unwrap();
    let res_should_be = "#2\n*1\n#2\n&1\n:1\n0\n".to_owned().into_bytes();
    let mut response = vec![0; res_should_be.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(
        response,
        res_should_be,
        "{}: With 3 k/v pair(s)",
        __func__!()
    );
}

#[tokio::test]
/// Test an MSET query with the wrong number of arguments
async fn test_mset_syntax_error() {
    let server = start_server(None).await;
    let mut stream = TcpStream::connect(ADDR).await.unwrap();
    let syntax_error = terrapipe::proc_query("MSET");
    stream.write_all(&syntax_error).await.unwrap();
    let mut response = vec![0; fresp::R_ACTION_ERR.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(
        response,
        fresp::R_ACTION_ERR.to_owned(),
        "{}: With one arg(s)",
        __func__!()
    );
    // Now we'll test it with two keys and one-value
    let syntax_error = terrapipe::proc_query("MSET x ex y");
    stream.write_all(&syntax_error).await.unwrap();
    let mut response = vec![0; fresp::R_ACTION_ERR.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(
        response,
        fresp::R_ACTION_ERR.to_owned(),
        "{}: With three arg(s)",
        __func__!()
    );
}

#[tokio::test]
/// Test an MUPDATE query with a single non-existing keys
async fn test_mupdate_single_okay() {
    let server = start_server(None).await;
    let mut stream = TcpStream::connect(ADDR).await.unwrap();
    let mut stream = set_values("x 100", 1, stream).await;
    let query = terrapipe::proc_query("MUPDATE x ex");
    stream.write_all(&query).await.unwrap();
    let res_should_be = "#2\n*1\n#2\n&1\n:1\n1\n".to_owned().into_bytes();
    let mut response = vec![0; res_should_be.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(response, res_should_be, "{}", __func__!());
}

#[tokio::test]
/// Test an MUPDATE query with a mixed set of outcomes
async fn test_mupdate_multiple_mixed() {
    let server = start_server(None).await;
    let mut stream = TcpStream::connect(ADDR).await.unwrap();
    let mut stream = set_values("x ex", 1, stream).await;
    let query = terrapipe::proc_query("MUPDATE x ex y why z zed");
    stream.write_all(&query).await.unwrap();
    let res_should_be = "#2\n*1\n#2\n&1\n:1\n1\n".to_owned().into_bytes();
    let mut response = vec![0; res_should_be.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(
        response,
        res_should_be,
        "{}: With 3 k/v pair(s)",
        __func__!()
    );
    // None of these keys exist, so we should get a 0
    let query = terrapipe::proc_query("MUPDATE y why z zed");
    stream.write_all(&query).await.unwrap();
    let res_should_be = "#2\n*1\n#2\n&1\n:1\n0\n".to_owned().into_bytes();
    let mut response = vec![0; res_should_be.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(
        response,
        res_should_be,
        "{}: With 2 k/v pair(s)",
        __func__!()
    );
}

#[tokio::test]
/// Test an MUPDATE query with the wrong number of arguments
async fn test_mupdate_syntax_error() {
    let server = start_server(None).await;
    let mut stream = TcpStream::connect(ADDR).await.unwrap();
    let syntax_error = terrapipe::proc_query("MUPDATE");
    stream.write_all(&syntax_error).await.unwrap();
    let mut response = vec![0; fresp::R_ACTION_ERR.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(
        response,
        fresp::R_ACTION_ERR.to_owned(),
        "{}: With one arg(s)",
        __func__!()
    );
    // Now we'll test it with two keys and one-value
    let syntax_error = terrapipe::proc_query("MUPDATE x ex y");
    stream.write_all(&syntax_error).await.unwrap();
    let mut response = vec![0; fresp::R_ACTION_ERR.len()];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(
        response,
        fresp::R_ACTION_ERR.to_owned(),
        "{}: With three arg(s)",
        __func__!()
    );
}
