//-------------------------------------------------------------------
// @author yangcancai

// Copyright (c) 2021 by yangcancai(yangcancai0112@gmail.com), All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//       https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

// @doc
//
// @end
// Created : 2021-11-09T07:47:46+00:00
//-------------------------------------------------------------------

use std::{
    borrow::Cow,
    sync::{RwLock, RwLockWriteGuard},
};

use rustler::resource::ResourceArc;
use rustler::{Binary, Encoder, Env, NifResult, OwnedBinary, Term};
use rustler::types::tuple::make_tuple;

use atoms::ok;
use rate_limiter::rate_limiter::Store;
use rate_limiter::RateLimiter;
use rate_limiter::rate_limiter::RateLimitResult;
use rate_limiter::rate_limiter::RateError;
use rate_limiter::rate_limiter::to_second;
// =================================================================================================
// resource
// =================================================================================================
struct NifrateLimiter {
    data: RateLimiter<Store>,
}
impl NifrateLimiter {
    // create
    fn new() -> Result<Self, String> {
        let store = Store::new();
        let rate = RateLimiter::new(store);
        Ok(NifrateLimiter { data: rate})
    }
    fn u8_to_string(&self, msg: &[u8]) -> String {
        let a = String::from_utf8_lossy(msg);
        match a {
            Cow::Owned(own_msg) => own_msg,
            Cow::Borrowed(b_msg) => b_msg.to_string(),
        }
    }
    fn clear(&mut self) {
        let store = Store::new();
        let rate = RateLimiter::new(store);
        self.data = rate;
    }
    fn delete(&mut self, key: &[u8]) {
        self.data.delete(self.u8_to_string(key));
    }
    fn run(
        &mut self,
        key: &[u8],
        burst: i64,
        count: i64,
        seconds: i64,
        quantity: i64,
    ) -> Result<RateLimitResult, RateError> {
        self.data
            .rate_limit(self.u8_to_string(key), burst, count, seconds, quantity)
    }
}
#[repr(transparent)]
struct NifrateLimiterResource(RwLock<NifrateLimiter>);

impl NifrateLimiterResource {
    fn write(&self) -> RwLockWriteGuard<'_, NifrateLimiter> {
    self.0.write().unwrap()
    }
}

impl From<NifrateLimiter> for NifrateLimiterResource{
    fn from(other: NifrateLimiter) -> Self {
        NifrateLimiterResource(RwLock::new(other))
    }
}

pub fn on_load(env: Env, _load_info: Term) -> bool {
    rustler::resource!(NifrateLimiterResource, env);
    true
}

// =================================================================================================
// api
// =================================================================================================

#[rustler::nif]
fn new(env: Env) -> NifResult<Term> {
    let rs = NifrateLimiter::new().map_err(|e| rustler::error::Error::Term(Box::new(e)))?;
    Ok((ok(), ResourceArc::new(NifrateLimiterResource::from(rs))).encode(env))
}
#[rustler::nif]
fn run<'a>(
    env: Env<'a>,
    resource: ResourceArc<NifrateLimiterResource>,
    key: LazyBinary<'a>,
    burst: i64,
    count: i64,
    seconds: i64,
    quantity: i64,
) -> NifResult<Term<'a>> {
    let rs = resource.write().run(&key, burst, count, seconds, quantity);
    if let Ok(rs) = rs{
        Ok(encode_rs(&rs, env))
    }else{
        Err(rustler::error::Error::BadArg)
    }
}
#[rustler::nif]
fn delete<'a>(env: Env<'a>, resource: ResourceArc<NifrateLimiterResource>, key: LazyBinary<'a>) -> NifResult<Term<'a>> {
    resource.write().delete(&key);
    Ok(ok().encode(env))
}
#[rustler::nif]
fn clear(env: Env, resource: ResourceArc<NifrateLimiterResource>) -> NifResult<Term> {
    resource.write().clear();
    Ok(ok().encode(env))
}
// =================================================================================================
// helpers
// =================================================================================================

/// Represents either a borrowed `Binary` or `OwnedBinary`.
///
/// `LazyBinary` allows for the most efficient conversion from an
/// Erlang term to a byte slice. If the term is an actual Erlang
/// binary, constructing `LazyBinary` is essentially
/// zero-cost. However, if the term is any other Erlang type, it is
/// converted to an `OwnedBinary`, which requires a heap allocation.
enum LazyBinary<'a> {
    Owned(OwnedBinary),
    Borrowed(Binary<'a>),
}

impl<'a> std::ops::Deref for LazyBinary<'a> {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        match self {
            Self::Owned(owned) => owned.as_ref(),
            Self::Borrowed(borrowed) => borrowed.as_ref(),
        }
    }
}

impl<'a> rustler::Decoder<'a> for LazyBinary<'a> {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        if term.is_binary() {
            Ok(Self::Borrowed(Binary::from_term(term)?))
        } else {
            Ok(Self::Owned(term.to_binary()))
        }
    }
}

fn encode_rs<'a>(rs: &RateLimitResult, env: Env<'a>) -> Term<'a> {
    let terms: Vec<_> = vec![rs.allowed.encode(env), rs.limit.encode(env),
    rs.remaining.encode(env),
    to_second(rs.reset_after).encode(env),
    to_second(rs.retry_after).encode(env)];
    make_tuple(env, terms.as_ref()).encode(env)
    }