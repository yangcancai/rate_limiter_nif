rate_limiter_nif
----

![CI](https://github.com/yangcancai/rate_limiter_nif/actions/workflows/ci.yml/badge.svg)

Required
-----
	$ rebar3 -v
	rebar 3.14.4 on Erlang/OTP 22 Erts 10.7.2.1

Build
-----

    $ make co

Eunit
-----

    $ make eunit

Common Test
-----

    $ make ct

Dialyzer
----

    $ make dialyzer

Test(dialyzer, eunit, ct)
----

    $ make test

Install
-----
```erlang
{deps, [
{rate_limiter_nif, {git, "https://github.com/yangcancai/rate_limiter_nif.git", {branch, "main"}}}
]}.
```
Example
-----

```erlang
(rate_limiter_nif@127.0.0.1)1> {ok, P} = rate_limiter_nif:new().
{ok,#Ref<0.3272858478.1929510913.232954>}
(rate_limiter_nif@127.0.0.1)2> rate_limiter_nif:run(P,<<"a">>,10,1,10,1).
{true,11,10,10,-1}
```