package org.corpus_tools.graphannis.info;

import org.bytedeco.javacpp.annotation.Namespace;
import org.bytedeco.javacpp.annotation.Platform;
import org.bytedeco.javacpp.annotation.Properties;
import org.bytedeco.javacpp.tools.Info;
import org.bytedeco.javacpp.tools.InfoMap;
import org.bytedeco.javacpp.tools.InfoMapper;

@Namespace("annis::api")
@Properties(target = "org.corpus_tools.graphannis.API",
  value =
  {
    @Platform(
      include =
      {
        "annis/api/corpusstoragemanager.h",
        "annis/api/admin.h",
        "annis/api/graphupdate.h"
      },
      link =
      {
        "re2", "boost_system", "boost_filesystem", "boost_thread", "humblelogging", "Vc", "annis"
      }
    )
    ,
    @Platform(value = "windows",
      link =
      {
        "re2", "humblelogging", "Vc", "annis"
      }
    ),
    @Platform(value = "macosx",
      link =
      {
        "re2", "boost_system-mt", "boost_filesystem-mt", "boost_thread-mt", "humblelogging", "Vc", "annis"
      }
    )
  })
public class AnnisApiInfo implements InfoMapper
{

  @Override
  public void map(InfoMap infoMap)
  {
    infoMap.put(new Info("std::vector<std::string>").pointerTypes("StringVector").define());
    infoMap.put(new Info("annis::Init").skip());
    infoMap.put(new Info("std::uint32_t").valueTypes("long"));
    infoMap.put(new Info("std::uint64_t").valueTypes("long"));
    infoMap.put(new Info("hash<annis::Annotation>").skip());
    infoMap.put(new Info("annis::api::GraphUpdate::getLastConsistentChangeID").skip());
    infoMap.put(new Info("std::vector<annis::api::UpdateEvent>").pointerTypes("UpdateEventList").define());
    infoMap.put(new Info("annis::api::GraphUpdate::getDiffs").skip());
  }

}
