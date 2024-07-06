defmodule Server do
  def handle() do
    Process.register(self(), :ss)
    loop()
  end
  def loop() do
    receive do
      {:call, pid} -> 
        Process.send(pid, :hi, [])
        IO.inspect(:call)
        loop()
      {msg, pid} ->
        # Process.unregister(:ss)
        IO.inspect(msg)
        Process.send(pid, :ack, [])
    end
  end
end

Server.handle

