defmodule Client do
  def handle() do
    pid = Node.spawn_link(:rust@fedora, IO, :inspect, [])
    IO.inspect([:pid, pid, 1111])
    Process.send(pid, {:hi, self()}, [])
    loop()
  end

  def loop() do
    receive do
       {:EXIT, _reason} ->
        Process.exit(self(), :normal)
      msg ->
        IO.inspect([:recv, msg])
        loop()
    end
  end
end

Client.handle

