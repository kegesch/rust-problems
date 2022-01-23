#![feature(async_closure)]

use futures::stream;
use futures::StreamExt;
use std::future::{self, Future};
use async_trait::async_trait;

#[derive(Debug)]
pub struct Number(usize);

#[async_trait]
pub trait Statement {
    async fn execute(&mut self, a: Number, b: Number) -> Number;
}

pub struct Multiplier;
impl Multiplier {
    pub fn mul(&mut self, a: Number, b: Number) -> impl Future<Output = Number> {
        future::ready(Number(a.0 * b.0))
    }
}

#[async_trait]
impl Statement for Multiplier {
    async fn execute(&mut self, a: Number, b: Number) -> Number {
        self.mul(a, b).await
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let numbers = vec![0,1,2,3].into_iter().map(|num| Number(num)).collect();
    let mut multiplier = Multiplier;
    let multiplier_ref = &mut multiplier;
    
    let stream = calculate(multiplier_ref, numbers).await;
    println!("{:#?}", stream);
   
    Ok(())
}

// This yields E507
async fn calculate_no_compile<S: Statement>(statement: &mut S, numbers: Vec<Number>) -> Vec<Number> {
    let stream = stream::iter(numbers).then(async move |num| {
        statement.execute(num, Number(2)).await
    })
    .collect::<Vec<_>>()
    .await;

    stream
}

// This works
// See also https://stackoverflow.com/a/62563511/8315238
async fn calculate<S: Statement>(statement: &mut S, numbers: Vec<Number>) -> Vec<Number> {
    let stream = stream::iter(numbers).fold((statement, vec![]), |(statement, mut vec), num| async move {
        vec.push(statement.execute(num, Number(2)).await);
        (statement, vec)
    })
    .await;

    stream.1
}