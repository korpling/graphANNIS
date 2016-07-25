package org.korpling.graphannis;

import org.bytedeco.javacpp.Loader;
import org.bytedeco.javacpp.Pointer;
import org.bytedeco.javacpp.annotation.Namespace;
import org.bytedeco.javacpp.annotation.Platform;
import org.bytedeco.javacpp.annotation.StdString;

@Platform(include="annis/api.h", link="ANNIS4")
@Namespace("annis")
public class API extends Pointer {

	static {
		Loader.load();
	}
	
	public native long count(@StdString String corpus, String queryAsJSON);
	
}
