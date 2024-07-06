defmodule Client do
  def handle() do
    pid = Node.spawn(:rust@fedora, IO, :inspect, [])
    IO.inspect([:pid, pid, 1111])
    Process.send(pid, {:hi, self()}, [])
    loop()
  end

  def loop() do
    receive do
      {:call, pid} -> 
        IO.inspect(:call)
        loop()
      msg ->
        IO.inspect([:recv, msg])
        loop()
    end
  end
end

Client.handle

