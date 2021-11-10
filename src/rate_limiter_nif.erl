-module(rate_limiter_nif).

%% API
-export([new/0, run/6, clear/1]).  %% new resource

            %% clear resource

%% Native library support
-export([load/0]).

-on_load load/0.

-opaque rate_limiter() :: reference().

-export_type([rate_limiter/0]).

new() ->
    not_loaded(?LINE).

-spec run(Ref :: rate_limiter(),
          Key :: binary(),
          Burst :: integer(),
          Count :: integer(),
          Seconds :: integer(),
          Quantity :: integer()) ->
             {Allowed :: boolean(),
              Limit :: integer(),
              Remain :: integer(),
              ResetAfter :: integer(),
              RetryAfter :: integer()}.
%% 桶大小=Burst+1
%% Count:时间seconds产生Count数量
%% Quantity:每次取走多少个令牌
%% Allowed=true 获取令牌成功
%% Allowed=false
%% 可以根据RetryAfter来间隔多少秒可以重新获取
%% ResetAfter代表多少秒后令牌桶会满
run(_Ref, _Key, _Burst, _Count, _Seconds, _Quantity) ->
    not_loaded(?LINE).

-spec clear(Ref :: rate_limiter()) -> ok.
clear(_Ref) ->
    not_loaded(?LINE).

%% @private
load() ->
    erlang:load_nif(
        filename:join(priv(), "librate_limiter"), none).

not_loaded(Line) ->
    erlang:nif_error({error, {not_loaded, [{module, ?MODULE}, {line, Line}]}}).

priv() ->
    case code:priv_dir(?MODULE) of
        {error, _} ->
            EbinDir =
                filename:dirname(
                    code:which(?MODULE)),
            AppPath = filename:dirname(EbinDir),
            filename:join(AppPath, "priv");
        Path ->
            Path
    end.
