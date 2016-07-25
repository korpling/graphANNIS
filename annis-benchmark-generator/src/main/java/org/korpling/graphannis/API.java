package org.korpling.graphannis;

import org.bytedeco.javacpp.Loader;
import org.bytedeco.javacpp.Pointer;
import org.bytedeco.javacpp.annotation.Namespace;
import org.bytedeco.javacpp.annotation.Platform;
import org.bytedeco.javacpp.annotation.StdString;
import org.korpling.annis.benchmark.generator.QueryToJSON;

import annis.ql.parser.AnnisParserAntlr;
import annis.ql.parser.QueryData;

@Platform(include="annis/api.h", link={"ANNIS4"}, 
	preload={"boost_system", "boost_filesystem", "boost_serialization"},
	preloadpath="/usr/lib/")
@Namespace("annis")
public class API extends Pointer {

	static {
		Loader.load();
	}
	
	public native long count(@StdString String corpus, String queryAsJSON);
	
	
	public static void main(String[] args) {
		API test = new API();
		
		AnnisParserAntlr parser = new AnnisParserAntlr();
		
		QueryData queryData = parser.parse(args[0], null);
		System.out.println("Count: " + test.count("pcc", QueryToJSON.serializeQuery(queryData)));
		
		test.close();
    }
	
}
