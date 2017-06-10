using Uno;
using Uno.Data;
using Uno.UX;
using Uno.Collections;
using Uno.Text;

using Fuse;
using Fuse.Elements;
using Fuse.Scripting;
using Fuse.Controls;

using Fuse.Controls.FuseTextRenderer;

public abstract class CrawlerAttribute
{
	public readonly string Key;

	public abstract string StringValue { get; }

	protected CrawlerAttribute(string key)
	{
		Key = key;
	}
}

public class CrawlerStringAttribute : CrawlerAttribute
{
	public readonly string Value;

	public override string StringValue { get { return Value; } }

	public CrawlerStringAttribute(string key, string val)
		: base(key)
	{
		Value = val;
	}
}

public class CrawlerNumberAttribute : CrawlerAttribute
{
	public readonly double Value;
	public override string StringValue { get { return Value + ""; } }
	public CrawlerNumberAttribute(string key, double val)
		: base(key)
	{
		Value = val;
	}
}

public class CrawlerNode
{

	public static int UnnamedNodeCount = 0;

	public readonly List<CrawlerNode> Children = new List<CrawlerNode>();
	public readonly List<CrawlerAttribute> Attributes = new List<CrawlerAttribute>();

	public bool Contains(string key)
	{
		foreach (var a in Attributes)
		{
			if (a.Key == key)
			{
				return true;
			}
		}
		return false;
	}

	public void Add(CrawlerAttribute attrib)
	{
		Attributes.Add(attrib);
	}

	public void Add(string key, float value)
	{
		Attributes.Add(new CrawlerNumberAttribute(key, (double)value));
	}

	public void Add(string key, double value)
	{
		Attributes.Add(new CrawlerNumberAttribute(key, value));
	}

	public void Add(string key, string value)
	{

		Attributes.Add(new CrawlerStringAttribute(key, (string)value));
	}

	public void Add(CrawlerNode child)
	{
		Children.Add(child);
	}

	public string GetString(string key)
	{
		foreach (var a in Attributes)
		{
			if (a.Key == key)
				return a.StringValue;
		}
		return null;
	}


	internal static Recti ConservativelySnapToCoveringIntegers(Rect r)
	{
		// To prevent translations from affecting the size, round off origin and size
		// separately. And because origin might be rounded down while size not, we need
		// to add one to the width to be sure.

		int2 origin = (int2)Math.Floor(r.LeftTop);
		int2 size = (int2)Math.Ceil(r.RightBottom - r.LeftTop + 0.01f);
		return new Recti(origin.X,	origin.Y,
						 origin.X + size.X + 1, origin.Y + size.Y + 1);
	}

	public CrawlerNode() { }

	public CrawlerNode(Element e)
	{
		var name = GetNodeName(e);
		var actualPos = e.WorldPosition;
		var size = e.ActualSize;
		Add("Name", name);

		Add("ActualPositionX", actualPos.X);
		Add("ActualPositionY", actualPos.Y);
		Add("File", e.FileName);
		Add("Line", e.LineNumber);

		if (e is Shape)
		{
			if (e is Circle) //aspect == 1
			{
				var renderBounds = e.ActualSize;
				var actualRenderSize = Math.Min(renderBounds.X, renderBounds.Y);
				Add("RenderWidth", actualRenderSize);
				Add("RenderHeight", actualRenderSize);

				//TODO: Might need to handle alignment here.

				var renderPosX = e.WorldPosition.X + (renderBounds.X / 2) - (actualRenderSize/2);
				var renderPosY = e.WorldPosition.Y + (renderBounds.Y / 2) - (actualRenderSize/2);

				Add("RenderPositionX", renderPosX);
				Add("RenderPositionY", renderPosY);
			}
		}
		else if (e is Image)
		{
			var i = (Image)e;
			var renderBounds = e.ActualSize;
			var actualRenderSize = Math.Min(renderBounds.X, renderBounds.Y);


			if (i.Source != null)
			{
				var source = i.Source;
				var aspect = source.Size.Y / source.Size.X;

				debug_log("Image Aspect: " + aspect);

				var renderWidth = 0;
				var renderHeight = 0;

				var renderPosX = 0;
				var renderPosY = 0;
				if (renderBounds.Y / renderBounds.X >= aspect)
				{
					renderWidth = (int)renderBounds.X;
					renderHeight = (int)(renderBounds.X * aspect);

					renderPosX = (int)actualPos.X;
					renderPosY = (int)(actualPos.Y + (size.Y / 2) - (renderHeight / 2));
				}
				else
				{
					renderWidth = (int)(renderBounds.Y * (1.0/aspect));
					renderHeight = (int)renderBounds.Y;

					renderPosX = (int)(actualPos.X + (size.X / 2) - (renderWidth / 2));
					renderPosY = (int)actualPos.Y;
				}


				Add("RenderPositionX", renderPosX);
				Add("RenderPositionY", renderPosY);

				Add("RenderWidth", renderWidth);
				Add("RenderHeight", renderHeight);
			}
			else
			{
				Add("RenderWidth", renderBounds.X);
				Add("RenderHeight", renderBounds.Y);
			}
		}
		else if (e is TextControl)
		{
			var tc = (TextControl)e;

			debug_log("Text bounds X: " + (int)tc.LastContentSize.X);
			debug_log("Text bounds Y: " + (int)tc.LastContentSize.Y);

			var renderWidth = (int)tc.LastContentSize.X;
			var renderHeight = (int)tc.LastContentSize.Y;

			Add("RenderWidth", renderWidth);
			Add("RenderHeight", renderHeight);
			Add("ActualWidth", Math.Max(renderWidth, size.X));
			Add("ActualHeight", Math.Max(renderHeight, size.Y));
		}

		if (!Contains("ActualWidth"))
			Add("ActualWidth", size.X);
		if (!Contains("ActualHeight"))
			Add("ActualHeight", size.Y);

		if (!Contains("RenderWidth") && !Contains("RenderHeight"))
		{
			var renderBounds = e.ActualSize;
			Add("RenderWidth", renderBounds.X);
			Add("RenderHeight", renderBounds.Y);
		}

		if (!Contains("RenderPositionX") && !Contains("RenderPositionY"))
		{
			var renderPosX = e.WorldPosition.X;
			var renderPosY = e.WorldPosition.Y;

			Add("RenderPositionX", renderPosX);
			Add("RenderPositionY", renderPosY);
		}
	}

	public CrawlerNode(RootViewport rv)
	{
		var actualPos = rv.WorldPosition;
		var size = rv.Size;
		//TODO(should perhaps add information about pixel density, although we can probably just assume non retina for now)
		var renderBounds = rv.Size;
		Add("Name", GetNodeName(rv));
		Add("File", rv.FileName);
		Add("Line", rv.LineNumber);
		//TODO: Consider not adding all these attributes
		Add("ActualWidth", size.X);
		Add("ActualHeight", size.Y);
		Add("RenderWidth", renderBounds.X);
		Add("RenderHeight", renderBounds.Y);
		Add("ActualPositionX", actualPos.X);
		Add("ActualPositionY", actualPos.Y);
		Add("RenderPositionX", actualPos.X);
		Add("RenderPositionY", actualPos.Y);
	}

	String GetNodeName(Node n)
	{
		if (!String.IsNullOrEmpty(n.Name))
		{
			return n.Name;
		}
		return "UnnamedNode_" + (UnnamedNodeCount++);
	}

	public override string ToString()
	{
		return GetString("Name") + ": W: " + GetString("ActualWidth") + ", H: " + GetString("ActualHeight") + ", RW: " + GetString("RenderWidth") + ", RH: " + GetString("RenderHeight");
	}
}

public class CrawlerResult
{
	CrawlerNode _root;
	public CrawlerNode Root
	{
		get { return _root; }
		set { _root = value; }
	}

	float2 _screenSize;
	public float2 ScreenSize
	{
		get { return _screenSize; }
		set { _screenSize = value; }
	}

	public CrawlerResult(CrawlerNode root, float2 size)
	{
		_root = root;
		_screenSize = size;
	}

	public override string ToString()
	{
		var sb = new StringBuilder();
		foreach (var n in _root.Children)
		{
			sb.AppendLine(n.ToString());
		}
		return sb.ToString();
	}
}


public partial class Crawler : Behavior
{
	public event Action<CrawlerResult> GotNewCrawlerResult;

	void OnGotNewCrawlerResult(CrawlerResult result)
	{
		var handler = GotNewCrawlerResult;
		if (handler != null)
		{
			handler(result);
		}
	}

	Visual FindRoot()
	{
		var currentNode = Parent as Visual;
		if (currentNode == null)
		{
			return null;
		}
		while (currentNode.Parent != null)
		{
			currentNode = currentNode.Parent;
		}
		return currentNode;
	}

	void VisitChildren(Visual node, CrawlerNode crawlerNode)
	{

		foreach (var c in node.Children)
		{
			//debug_log("Child: " + c);
			if (c is GUITestOracle || c is Crawler) {
				continue;
			}
			if (c is Element)
			{
				var crawlerElement = new CrawlerNode((Element)c);
				crawlerNode.Add(crawlerElement);
				VisitChildren((Element)c, crawlerElement);
			}
			else if (c is RootViewport)
			{
				var crawlerViewport = new CrawlerNode((RootViewport)c);
				crawlerNode.Add(crawlerViewport);
				VisitChildren((Visual)c, crawlerViewport);
			}
			//TODO(need to cover NodeGroupBase)
		}
	}

	public CrawlerResult Crawl()
	{
		CrawlerNode.UnnamedNodeCount = 0;

		var root = FindRoot();
		if (root == null) {
			return null;
		}
		//debug_log("Type of root: " + root);
		//TODO check this downcast
		var rootCrawlerNode = new CrawlerNode((RootViewport)root);
		VisitChildren(root, rootCrawlerNode);

		var screenSize = float2(0);
		if (root is RootViewport)
		{
			screenSize = ((RootViewport)root).Size;
		}
		else if (root is Element)
		{
			screenSize = ((Element)root).ActualSize;
		}
		else
		{
			debug_log("ERROR: We have a weird root node :S (not a RootViewport or Element)");
			throw new Exception("We have a weird root node :S (not a RootViewport or Element)");
		}

		var result = new CrawlerResult(rootCrawlerNode, screenSize);

		return result;
	}

	void ParentPlaced(object sender, PlacedArgs args)
	{
		debug_log("Parent placed");
		var result = Crawl();
		if (result != null)
		{
			OnGotNewCrawlerResult(result);
		}
	}

	protected override void OnRooted()
	{
		base.OnRooted();

		/*var e = Parent as Element;
		if (e != null)
		{
			e.Placed += ParentPlaced;
		}*/
	}

	protected override void OnUnrooted()
	{
		base.OnUnrooted();
	}
}
