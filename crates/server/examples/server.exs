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
      msg ->
        Process.unregister(:ss)
        IO.inspect(msg)
    end
  end
end

Server.handle

