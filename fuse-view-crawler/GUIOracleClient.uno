using Fuse;
using Uno;
using Uno.Collections;
using Uno.Net;
using Uno.Net.Sockets;
using Uno.Text;
using Uno.Text.Encoding;
using Uno.Data.Json;

public class GUIOracleClient
{
	Socket _socket;

	public GUIOracleClient()
	{
		UpdateManager.AddAction(this.Recv);
		//Recv();
	    Connect();
	}

	public bool Connect()
	{
		var endPoint = new IPEndPoint(IPAddress.Parse("127.0.0.1"), 12345);
		var localEndPoint = new IPEndPoint(IPAddress.Parse("127.0.0.1"), 12344);
		try
		{
			_socket = new Socket(AddressFamily.InterNetwork, SocketType.Stream, ProtocolType.Tcp);
			_socket.Bind(localEndPoint);
			_socket.Connect(endPoint);
			debug_log("Connected");
			return true;
		}
		catch (Exception e)
		{
			_socket = null;
	//		debug_log("Exception: " + e.Message);
//			debug_log("Could not connect to " + endPoint.ToString());
			return false;
		}


	}

	public Action<int> ReceivedRequest;

	void OnReceivedRequest(int i)
	{
		var handler = ReceivedRequest;
		if (handler != null)
		{
			handler(i);
		}
	}

	List<byte> _bytes = new List<byte>();

	int _bufferPos = 0;

	bool TryGetPosition(byte of, out int index)
	{
		index = -1;
		var c = 0;
		for (var i = 0; i < _bytes.Count; i++)
		{
			var b = _bytes[i + _bufferPos];
			if (b == of)
			{
				index = c;
				return true;
			}
			c += 1;
		}
		return false;
	}

	string DrainStringTo(int pos)
	{
		var bytes = new byte[pos];
		for (var i = 0; i < pos; i++)
		{
			bytes[i] = _bytes[_bufferPos];
			_bufferPos += 1;
		}
		return Utf8.GetString(bytes);
	}

	void DrainMessage()
	{
		for (int i = 0; i < _bufferPos; i++)
		{
			_bytes.RemoveAt(0);
		}
		_bufferPos = 0;
	}

	List<Message> TryDecodeMessages()
	{
		var ret = new List<Message>();
		_bufferPos = 0;

		var buffer_str = Utf8.GetString(_bytes.ToArray());
		debug_log("BufferStr: " + buffer_str);

		while (true)
		{
			var i = -1;
			if (TryGetPosition((byte)'\n', out i))
			{
				debug_log("I: " + i);
				var messageType = DrainStringTo(i);
				DrainStringTo(1); //draining the \n
				debug_log("Message type: " + messageType);
				var i2 = -1;
				if (TryGetPosition((byte)'\n', out i2))
				{
					debug_log("I2: " + i2);
					var messageLengthString = DrainStringTo(i2);
					DrainStringTo(1); //draining the \n
					debug_log("MessageLengthString: " + messageLengthString);
					var messageLength = Int.Parse(messageLengthString);
					var data = DrainStringTo(messageLength);

					debug_log("Data: " + data);

					var msgType = MessageType.None;

					var msgTypeInt = Int.Parse(messageType);
					if (msgTypeInt == 0)
					{
						msgType = MessageType.LayoutData;
					}
					else if (msgTypeInt == 1)
					{
						msgType = MessageType.RequestLayoutData;
					}
					var message = new Message(msgType, data);
					DrainMessage();
					ret.Add(message);
					debug_log("WE MADE A MESSAGE");
				}
				else
				{
					_bufferPos = 0;
					break;
				}
			}
			else
			{
				_bufferPos = 0;
				break;
			}
		}
		return ret;
	}


	public void Recv()
	{
		if (_socket == null)
		{
			Connect();
			return;
		}


		var available = _socket.Available;
		debug_log("We are reading: " + available);
		if (available > 0)
		{
			var received = new byte[available];
			var bytesReceived = _socket.Receive(received);
			if (bytesReceived > 0)
			{
				_bytes.AddRange(received);
				debug_log("We got some bytes: " + bytesReceived);
				foreach (var message in TryDecodeMessages())
				{
					var msg_data = message.Data;
					var reader = JsonReader.Parse(msg_data);

					var id_str = reader["json_string"].AsString();
					var id = Int.Parse(id_str);

					OnReceivedRequest(id);
				}
			}
		}
	}

	public void Send(MessageType type, string data)
	{
		if (_socket == null)
		{
			Connect();
			return;
		}
		var available = _socket.Available;
		//debug_log("Available: " + available);
		var message = new Message(type, data);
		//debug_log("Connected?: " + _socket.Connected);
		//debug_log("We are abou to write to socket: " + message);
		var bytes = message.GetBytes();
		debug_log("We are about to send: " + bytes.Length + " bytes");
		_socket.Send(bytes);

		//debug_log("Received bytes: " + bytesReceived);
		//debug_log("      available: " + _socket.Available);

	}
}
