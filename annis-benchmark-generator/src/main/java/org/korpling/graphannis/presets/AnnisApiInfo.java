package org.korpling.graphannis.presets;

import org.bytedeco.javacpp.annotation.Namespace;
import org.bytedeco.javacpp.annotation.Platform;
import org.bytedeco.javacpp.annotation.Properties;
import org.bytedeco.javacpp.tools.Info;
import org.bytedeco.javacpp.tools.InfoMap;
import org.bytedeco.javacpp.tools.InfoMapper;

@Namespace("annis")
@Properties(target="org.korpling.graphannis.Annis",
    value={@Platform(
        include="annis/api.h", 
        link={"boost_system", "boost_filesystem", "boost_serialization", "humblelogging", "ANNIS4"}
        )})
public class AnnisApiInfo implements InfoMapper
{

  @Override
  public void map(InfoMap infoMap)
  {
  }

}
