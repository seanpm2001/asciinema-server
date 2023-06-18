defmodule Asciinema.Streaming.ProducerHandler.Raw do
  @behaviour Asciinema.Streaming.ProducerHandler

  def init, do: %{first: true, start_time: nil}

  def parse({payload, _}, %{first: true} = state) do
    size = size_from_resize_seq(payload) || size_from_script_start_message(payload)

    commands =
      case size do
        nil ->
          [feed: {0.0, payload}]

        size ->
          [reset: %{size: size, init: payload, time: 0.0}]
      end

    state = %{state | first: false, start_time: Timex.now()}

    {:ok, commands, state}
  end

  def parse({payload, _}, state) do
    time = Timex.diff(Timex.now(), state.start_time, :microsecond) / 1_000_000

    {:ok, [feed: {time, payload}], state}
  end

  defp size_from_resize_seq(text) do
    with [_, rows, cols] <- Regex.run(~r/\x1b\[8;(\d+);(\d+)t/, text) do
      {String.to_integer(cols), String.to_integer(rows)}
    end
  end

  defp size_from_script_start_message(text) do
    with [_, cols, rows] <- Regex.run(~r/\[.*COLUMNS="(\d{1,3})" LINES="(\d{1,3})".*\]/, text) do
      {String.to_integer(cols), String.to_integer(rows)}
    end
  end
end
