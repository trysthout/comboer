defmodule Client do
  def handle() do
    Node.connect(:rust@fedora);
    Process.send({:name, :rust@fedora}, {:hi, self()}, [])
    receive do
      m -> IO.inspect(m)
    end
  end
end

Client.handle

