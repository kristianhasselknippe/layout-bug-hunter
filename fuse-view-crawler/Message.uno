using Fuse;
using Uno;
using Uno.Text;
using Uno.Text.Encoding;
using Uno.Collections;

public enum MessageType
{
	LayoutData = 0,
	RequestLayoutData = 1,
	None = -1
}

public class Message
{
	MessageType _type;
	public MessageType Type
	{
		get { return _type; }
		set { _type = value; }
	}

	string _data;
	public string Data
	{
		get { return _data; }
		set { _data = value; }
	}

	public Message(MessageType type, string data)
	{
		_type = type;
		_data = data;
	}

	public byte[] GetBytes()
	{
		var sb = new StringBuilder();
		sb.Append("" + ((int)_type) + "\n");
		sb.Append(_data.Length + "\n");
		sb.Append(_data);
		var ret = sb.ToString();
		//debug_log("Msg: " + ret);
		//debug_log("Linefeed:" + ret[1]);
		return Utf8.GetBytes(ret);
	}

	public override string ToString() {
		return _data;
	}
}
