#pragma once

#include <annis/db.h>
#include <boost/optional.hpp>

#include <map>

namespace annis {

class RelANNISLoader
{
public:
  RelANNISLoader(DB& db);

  bool load(std::string dirPath);

  static bool loadRelANNIS(DB& db, std::string dirPath);
private:
  DB& db;

private:
  std::string loadRelANNISCorpusTab(std::string dirPath,
                                    std::map<uint32_t, uint32_t> &corpusByPreOrder,
                                    std::map<std::uint32_t, std::string> &corpusIDToName,
    bool isANNIS33Format);
  bool loadRelANNISNode(std::string dirPath, std::map<std::uint32_t, std::string> &corpusIDToName,
                        std::multimap<uint32_t, nodeid_t>& nodesByCorpusID,
                        std::string toplevelCorpusName,
                        bool isANNIS33Format);

  bool loadRelANNISRank(const std::string& dirPath,
                        const std::map<uint32_t, std::shared_ptr<WriteableGraphStorage> > &componentToGS,
                        bool isANNIS33Format);

  bool loadEdgeAnnotation(const std::string& dirPath,
                          const std::map<uint32_t, std::shared_ptr<WriteableGraphStorage> > &pre2GS,
                          const std::map<std::uint32_t, Edge>& pre2Edge,
                          bool isANNIS33Format);

  void loadCorpusAnnotation(const std::string& dirPath,
                            std::multimap<uint32_t, Annotation> &corpusId2Annos, bool isANNIS33Format);

  void addSubCorpora(std::string toplevelCorpusName, std::map<uint32_t, uint32_t> &corpusByPreOrder,
                     std::map<uint32_t, std::string> &corpusIDToName, std::multimap<uint32_t, nodeid_t>& nodesByCorpusID,
                     std::multimap<uint32_t, Annotation> &corpusId2Annos);

  ComponentType componentTypeFromShortName(std::string shortType);
};

}
