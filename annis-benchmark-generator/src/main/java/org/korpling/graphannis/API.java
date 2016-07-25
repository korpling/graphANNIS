package org.korpling.graphannis;

import org.bytedeco.javacpp.Loader;
import org.bytedeco.javacpp.Pointer;
import org.bytedeco.javacpp.annotation.Namespace;
import org.bytedeco.javacpp.annotation.Platform;
import org.bytedeco.javacpp.annotation.StdString;
import org.korpling.annis.benchmark.generator.QueryToJSON;

import annis.ql.parser.AnnisParserAntlr;
import annis.ql.parser.QueryData;

@Platform(include="annis/api.h", link={"boost_system", "boost_filesystem", "boost_serialization", "humblelogging", "ANNIS4"})
@Namespace("annis")
public class API extends Pointer {

	static {
		Loader.load();
	}
	
	public API()
	{
	  allocate();
	}
	
	public native void allocate(); 
	
	public native long count(@StdString String corpus, String queryAsJSON);
	
	
	public static void main(String[] args) {
		
	  API test = new API();
		
		AnnisParserAntlr parser = new AnnisParserAntlr();
		
		QueryData queryData = parser.parse(args[0], null);
		System.out.println("Count: " + test.count("pcc2", QueryToJSON.serializeQuery(queryData)));
		
		test.close();
    }
	
}
