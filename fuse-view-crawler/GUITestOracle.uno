using Fuse;
using Fuse.Elements;
using Uno.Collections;

public class GUITestOracle : Behavior
{
	GUIOracleClient _client;
	Crawler _crawler;

	protected override void OnRooted()
	{
		debug_log("::::::::::::::::::::::::::::::::::::::::::");
		_client = new GUIOracleClient();
		_crawler = new Crawler();
		Parent.Children.Add(_crawler);
		_crawler.GotNewCrawlerResult += GotNewCrawlerResult;
		_client.ReceivedRequest += ReceivedRequest;

		_crawler.Crawl();

		if (Parent is Element) {
			var e = (Element)Parent;
			e.Placed += OnPlaced;
		}
	}

	void OnPlaced(object arg, PlacedArgs sender)
	{
		_crawler.Crawl();
	}

	void ReceivedRequest(int id)
	{
		debug_log("Client received request");
		var result = _crawler.Crawl();
		GotNewCrawlerResult(result);
	}

	protected override void OnUnrooted()
	{
		_crawler.GotNewCrawlerResult -= GotNewCrawlerResult;
	}

	void GotNewCrawlerResult(CrawlerResult result)
	{
		var jsonWriter = new CrawlerResultJsonWriter();
		var jsonData = jsonWriter.Serialize(result, 0);
		//TODO: use an actual ID here ^^
		debug_log("JSON: " + jsonData);
		_client.Send(MessageType.LayoutData, jsonData);
	}
}
