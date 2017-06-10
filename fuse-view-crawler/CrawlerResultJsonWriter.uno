using Uno;
using Uno.Data;
using Uno.UX;
using Uno.Collections;
using Uno.Text;

using Fuse;
using Fuse.Elements;
using Fuse.Scripting;

public static class JsonHelper
{
	public static string IndentString(int indent)
	{
		return "";
		var sb = new StringBuilder();
		for (var i = 0; i < indent; i++)
		{
			sb.Append('\t');
		}
		return sb.ToString();
	}
}


public class CrawlerResultJsonWriter
{

	int _indent = 0;
	StringBuilder _sb;

	public CrawlerResultJsonWriter()
	{
		_sb = new StringBuilder();
	}

	void AppendString(string str, bool indent = true)
	{
		if (indent)
		{
			/*for (int i = 0; i < _indent; i++)
				_sb.Append('\t');*/
		}

		_sb.Append(str);
	}

	void AppendLine(string str, bool indent = true)
	{
		AppendString(str + "\n", indent);
	}

	void NewLine(bool indent = true)
	{
		AppendString("\n", indent);
	}

	void AppendKeyValue<T>(string key, T val, bool comma = true)
	{
		if (val is string)
		{
			AppendString("\"" + key + "\": " + "\"" + val + "\"");
		}
		else
		{
			AppendString("\"" + key + "\": " + val);
		}
		if (comma)
		{
			AppendString(",\n", false);
		}
		else
		{
			NewLine(false);
		}
	}

	void PushArray()
	{
		AppendLine("[");
		_indent++;
	}

	void PopArray()
	{
		_indent--;
		AppendString("]");
	}

	void PushObject()
	{
		AppendLine("{");
		_indent++;
	}

	void PopObject()
	{
		_indent--;
		AppendString("}");
	}

	void WriteElementContent(CrawlerNode node)
	{
		for (var i = 0; i < node.Attributes.Count; i++)
		{
			var a = node.Attributes[i];
			if (a is CrawlerStringAttribute)
			{
				var sa = (CrawlerStringAttribute)a;
				AppendKeyValue(sa.Key, sa.Value);
			}
			else if (a is CrawlerNumberAttribute)
			{
				var na = (CrawlerNumberAttribute)a;
				AppendKeyValue(na.Key, na.Value);
			}
		}
	}

	void WriteNode(CrawlerNode node)
	{
		PushObject();

		WriteElementContent((CrawlerNode)node);
		AppendString("\"Children\":");
		NewLine();
		PushArray();
		var c = 0;
		foreach (var n in node.Children)
		{
			WriteNode(n);
			if (c < node.Children.Count - 1)
			{
				AppendString(",", false);
			}
			c++;
			NewLine();
		}
		PopArray();
		NewLine();

		PopObject();
	}

	public string Serialize(CrawlerResult crawlerResult, int id)
	{
		PushObject();
		var screenSize = crawlerResult.ScreenSize;
		AppendString("\"ScreenSize\": { \"W\":"  + screenSize.X + ", \"H\":" + screenSize.Y + "},\n", false);
		AppendString("\"Id\": " + id +  ",\n", false);
		AppendString("\"Nodes\": ", false);
		WriteNode(crawlerResult.Root);
		PopObject();
		return _sb.ToString();
	}




}
