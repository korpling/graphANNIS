#include "relannisloader.h"

#include <annis/util/helper.h>

#include <string>
#include <map>
#include <boost/filesystem.hpp>
#include <boost/format.hpp>
#include <boost/optional.hpp>
#include <humblelogging/api.h>
#include <humblelogging/logger.h>

HUMBLE_LOGGER(logger, "annis4");


#if defined(__linux__) || defined(__linux) || defined(linux) || defined(__gnu_linux__)
#include <malloc.h>
#endif // LINUX

using namespace annis;
using namespace std;

RelANNISLoader::RelANNISLoader(DB& db)
  : db(db)
{

}

bool RelANNISLoader::load(string dirPath)
{
  db.clear();

  // check if this is the ANNIS 3.3 import format
  bool isANNIS33Format = false;
  if(boost::filesystem::is_regular_file(dirPath + "/annis.version"))
  {
    ifstream inVersion;
    inVersion.open(dirPath + "/annis.version", ifstream::in);
    if (inVersion.good())
    {
      std::string versionStr;
      inVersion >> versionStr;
      if(versionStr == "3.3")
      {
        isANNIS33Format = true;
      }
    }
  }

  std::map<std::uint32_t, std::uint32_t> corpusByPreOrder;
  map<uint32_t, std::string> corpusIDToName;
  std::string toplevelCorpusName = loadRelANNISCorpusTab(dirPath, corpusByPreOrder,
                                                         corpusIDToName, isANNIS33Format);
  if(toplevelCorpusName.empty())
  {
    std::cerr << "Could not find toplevel corpus name" << std::endl;
    return false;
  }

  multimap<uint32_t, nodeid_t> nodesByCorpusID;

  if(loadRelANNISNode(dirPath, corpusIDToName, nodesByCorpusID, toplevelCorpusName, isANNIS33Format) == false)
  {
    return false;
  }

  string componentTabPath = dirPath + "/component" + (isANNIS33Format ? ".annis" : ".tab");
  HL_INFO(logger, (boost::format("loading %1%") % componentTabPath).str());

  ifstream in;
  vector<string> line;

  in.open(componentTabPath, ifstream::in);
  if(!in.good()) return false;

  map<uint32_t, std::shared_ptr<WriteableGraphStorage>> componentToGS;
  while((line = Helper::nextCSV(in)).size() > 0)
  {
    uint32_t componentID = Helper::uint32FromString(line[0]);
    if(line[1] != "NULL")
    {
      ComponentType ctype = componentTypeFromShortName(line[1]);
      std::shared_ptr<WriteableGraphStorage> gs = db.createWritableGraphStorage(ctype, line[2], line[3]);
      componentToGS[componentID] = gs;
    }
  }

  in.close();

  bool result = loadRelANNISRank(dirPath, componentToGS, isANNIS33Format);

  std::multimap<uint32_t, Annotation> corpusId2Annos;
  loadCorpusAnnotation(dirPath, corpusId2Annos, isANNIS33Format);

  // add all (sub-) corpora and documents as explicit nodes
  addSubCorpora(toplevelCorpusName, corpusByPreOrder, corpusIDToName, nodesByCorpusID, corpusId2Annos);

  // construct the complex indexes for all components
  db.optimizeAll();

  HL_INFO(logger, "Updating statistics");
  db.nodeAnnos.calculateStatistics(db.strings);

  #if defined(__linux__) || defined(__linux) || defined(linux) || defined(__gnu_linux__)
  malloc_trim(0);
  #endif // LINUX

  HL_INFO(logger, "Finished loading relANNIS");
  return result;
}

bool RelANNISLoader::loadRelANNIS(DB &db, std::string dirPath)
{
  RelANNISLoader loader(db);
  return loader.load(dirPath);
}

std::string RelANNISLoader::loadRelANNISCorpusTab(string dirPath,
                                                  std::map<std::uint32_t, std::uint32_t>& corpusByPreOrder,
                                                  std::map<uint32_t, std::string>& corpusIDToName,
                                                  bool isANNIS33Format)
{
  std::string toplevelCorpus = "";

  string corpusTabPath = dirPath + "/corpus" + (isANNIS33Format ? ".annis" : ".tab");
  HL_INFO(logger, (boost::format("loading %1%") % corpusTabPath).str());

  ifstream in;
  in.open(corpusTabPath, ifstream::in);
  if(!in.good())
  {
    string msg = "Can't find corpus";
    msg += (isANNIS33Format ? ".annis" : ".tab");
    HL_ERROR(logger, msg);
    return "";
  }

  vector<string> line;
  while((line = Helper::nextCSV(in)).size() > 0)
  {
    std::uint32_t corpusID = Helper::uint32FromString(line[0]);
    corpusIDToName[corpusID] = line[1];

    std::string name = line[1];
    std::string type = line[2];
    std::uint32_t preOrder = Helper::uint32FromString(line[4]);
    //std::uint32_t postOrder= Helper::uint32FromString(line[5]);

    if(type == "CORPUS" && preOrder == 0)
    {
      toplevelCorpus = name;
    }
    else if(type == "DOCUMENT")
    {
      // TODO: do not only add documents but also sub-corpora
      corpusByPreOrder[preOrder] = corpusID;
    }
  }
  return toplevelCorpus;
}

bool RelANNISLoader::loadRelANNISNode(string dirPath,
                                      map<uint32_t, std::string>& corpusIDToName,
                                      multimap<uint32_t, nodeid_t> &nodesByCorpusID,
                                      std::string toplevelCorpusName,
                                      bool isANNIS33Format)
{
  typedef multimap<TextProperty, uint32_t>::const_iterator TextPropIt;

  // maps a token index to an node ID
  map<TextProperty, uint32_t> tokenByIndex;

  // map the "left" value to the nodes it belongs to
  multimap<TextProperty, nodeid_t> leftToNode;
  // map the "right" value to the nodes it belongs to
  multimap<TextProperty, nodeid_t> rightToNode;
  // map as node to it's "left" value
  map<nodeid_t, uint32_t> nodeToLeft;
  // map as node to it's "right" value
  map<nodeid_t, uint32_t> nodeToRight;

  // maps a character position to it's token
  map<TextProperty, nodeid_t> tokenByTextPosition;

  map<nodeid_t, string> missingSegmentationSpan;

  string nodeTabPath = dirPath + "/node" + (isANNIS33Format ? ".annis" : ".tab");
  HL_INFO(logger, (boost::format("loading %1%") % nodeTabPath).str());

  ifstream in;
  in.open(nodeTabPath, ifstream::in);
  if(!in.good())
  {
    std::string msg = "Can't find node";
    msg += isANNIS33Format ? ".annis" : ".tab";
    HL_ERROR(logger, msg);
    return false;
  }

  std::list<std::pair<NodeAnnotationKey, uint32_t>> annoList;

  {
    vector<string> line;
    while((line = Helper::nextCSV(in)).size() > 0)
    {
      uint32_t nodeNr;
      stringstream nodeNrStream(line[0]);
      nodeNrStream >> nodeNr;

      bool hasSegmentations = isANNIS33Format || line.size() > 10;
      string tokenIndexRaw = line[7];
      uint32_t textID = Helper::uint32FromString(line[1]);
      uint32_t corpusID = Helper::uint32FromString(line[2]);
      string layer = line[3];

      std::string docName = corpusIDToName[corpusID];
      nodesByCorpusID.insert({corpusID, nodeNr});

      Annotation nodeNameAnno;
      nodeNameAnno.ns = db.strings.add(annis_ns);
      nodeNameAnno.name = db.strings.add(annis_node_name);
      nodeNameAnno.val = db.strings.add(toplevelCorpusName + "/" + docName + "#" + line[4]);
      annoList.push_back(std::pair<NodeAnnotationKey, uint32_t>({nodeNr, nodeNameAnno.name, nodeNameAnno.ns }, nodeNameAnno.val));


      Annotation nodeTypeAnno;
      nodeTypeAnno.ns = db.strings.add(annis_ns);
      nodeTypeAnno.name = db.strings.add(annis_node_type);
      nodeTypeAnno.val = db.strings.add("node");
      annoList.push_back(std::pair<NodeAnnotationKey, uint32_t>({nodeNr, nodeTypeAnno.name, nodeTypeAnno.ns }, nodeTypeAnno.val));

      if(!layer.empty() && layer != "NULL")
      {
        Annotation layerAnno;
        layerAnno.ns = db.getNamespaceStringID();
        layerAnno.name = db.strings.add("layer");
        layerAnno.val = db.strings.add(layer);
        annoList.push_back(std::pair<NodeAnnotationKey, uint32_t>({nodeNr, layerAnno.name, layerAnno.ns }, layerAnno.val));
      }

      TextProperty left;
      left.segmentation = "";
      left.val = Helper::uint32FromString(line[5]);
      left.textID = textID;
      left.corpusID = corpusID;

      TextProperty right;
      right.segmentation = "";
      right.val = Helper::uint32FromString(line[6]);
      right.textID = textID;
      right.corpusID = corpusID;

      leftToNode.insert(pair<TextProperty, uint32_t>(left, nodeNr));
      rightToNode.insert(pair<TextProperty, uint32_t>(right, nodeNr));
      nodeToLeft[nodeNr] = left.val;
      nodeToRight[nodeNr] = right.val;

      if(tokenIndexRaw != "NULL")
      {
        string span = hasSegmentations ? line[12] : line[9];

        Annotation tokAnno;
        tokAnno.ns = db.strings.add(annis_ns);
        tokAnno.name = db.strings.add(annis_tok);
        tokAnno.val = db.strings.add(span);
        annoList.push_back(std::pair<NodeAnnotationKey, uint32_t>({nodeNr, tokAnno.name, tokAnno.ns }, tokAnno.val));

        TextProperty index;
        index.segmentation = "";
        index.val = Helper::uint32FromString(tokenIndexRaw);
        index.textID = textID;
        index.corpusID = corpusID;

        tokenByIndex.insert({index, nodeNr});

        TextProperty textPos;
        textPos.segmentation = "";
        textPos.textID = textID;
        textPos.corpusID = corpusID;
        for(uint32_t i=left.val; i <= right.val; i++)
        {
          textPos.val = i;
          tokenByTextPosition.insert({textPos, nodeNr});
        }

      } // end if token
      else if(hasSegmentations)
      {
        std::string segmentationName = isANNIS33Format ? line[11] : line[8];
        if(segmentationName != "NULL")
        {
          size_t segIndex = isANNIS33Format ? Helper::uint32FromString(line[10]) : Helper::uint32FromString(line[9]);

          if(isANNIS33Format)
          {
            // directly add the span information
            Annotation tokAnno;
            tokAnno.ns = db.strings.add(annis_ns);
            tokAnno.name = db.strings.add(annis_tok);
            tokAnno.val = db.strings.add(line[12]);
            annoList.push_back(std::pair<NodeAnnotationKey, uint32_t>({nodeNr, tokAnno.name, tokAnno.ns }, tokAnno.val));
          }
          else
          {
            // we need to get the span information from the node_annotation file later
            missingSegmentationSpan[nodeNr] = segmentationName;
          }

          // also add the specific segmentation index
          TextProperty index;
          index.segmentation = segmentationName;
          index.val = segIndex;
          index.textID = textID;
          index.corpusID = corpusID;

          tokenByIndex[index] = nodeNr;

        } // end if node has segmentation info
      } // endif if check segmentations

    }

    in.close();
  }

  // TODO: cleanup, better variable naming and put this into it's own function
  // iterate over all token by their order, find the nodes with the same
  // text coverage (either left or right) and add explicit ORDERING, LEFT_TOKEN and RIGHT_TOKEN edges
  if(!tokenByIndex.empty())
  {
    HL_INFO(logger, "calculating the automatically generated ORDERING, LEFT_TOKEN and RIGHT_TOKEN edges");
    std::shared_ptr<WriteableGraphStorage> gsLeft = db.createWritableGraphStorage(ComponentType::LEFT_TOKEN, annis_ns, "");
    std::shared_ptr<WriteableGraphStorage> gsRight = db.createWritableGraphStorage(ComponentType::RIGHT_TOKEN, annis_ns, "");

    map<TextProperty, uint32_t>::const_iterator tokenIt = tokenByIndex.begin();
    uint32_t lastTextID = numeric_limits<uint32_t>::max();
    uint32_t lastCorpusID = numeric_limits<uint32_t>::max();
    uint32_t lastToken = numeric_limits<uint32_t>::max();

    std::string lastSegmentation = "";

    while(tokenIt != tokenByIndex.end())
    {
      uint32_t currentToken = tokenIt->second;
      uint32_t currentTextID = tokenIt->first.textID;
      uint32_t currentCorpusID = tokenIt->first.corpusID;
      string currentSegmentation = tokenIt->first.segmentation;

      if(currentSegmentation == "")
      {
        // find all nodes that start together with the current token
        TextProperty currentTokenLeft;
        currentTokenLeft.segmentation = "";
        currentTokenLeft.textID = currentTextID;
        currentTokenLeft.corpusID = currentCorpusID;
        currentTokenLeft.val = nodeToLeft[currentToken];

        pair<TextPropIt, TextPropIt> leftAlignedNodes = leftToNode.equal_range(currentTokenLeft);
        for(TextPropIt itLeftAligned=leftAlignedNodes.first; itLeftAligned != leftAlignedNodes.second; itLeftAligned++)
        {
          gsLeft->addEdge(Init::initEdge(itLeftAligned->second, currentToken));
          gsLeft->addEdge(Init::initEdge(currentToken, itLeftAligned->second));
        }

        // find all nodes that end together with the current token
        TextProperty currentTokenRight;
        currentTokenRight.segmentation = "";
        currentTokenRight.textID = currentTextID;
        currentTokenRight.corpusID = currentCorpusID;
        currentTokenRight.val = nodeToRight[currentToken];


        pair<TextPropIt, TextPropIt> rightAlignedNodes = rightToNode.equal_range(currentTokenRight);
        for(TextPropIt itRightAligned=rightAlignedNodes.first;
            itRightAligned != rightAlignedNodes.second;
            itRightAligned++)
        {
          gsRight->addEdge(Init::initEdge(itRightAligned->second, currentToken));
          gsRight->addEdge(Init::initEdge(currentToken, itRightAligned->second));
        }
      }

      std::shared_ptr<WriteableGraphStorage> gsOrder = db.createWritableGraphStorage(ComponentType::ORDERING,
                                                                                     annis_ns, currentSegmentation);

      // if the last token/text value is valid and we are still in the same text
      if(tokenIt != tokenByIndex.begin()
         && currentCorpusID == lastCorpusID
         && currentTextID == lastTextID
         && currentSegmentation == lastSegmentation)
      {
        // we are still in the same text
        uint32_t nextToken = tokenIt->second;
        // add ordering between token
        gsOrder->addEdge(Init::initEdge(lastToken, nextToken));

      } // end if same text

      // update the iterator and other variables
      lastTextID = currentTextID;
      lastCorpusID = currentCorpusID;
      lastToken = tokenIt->second;
      lastSegmentation = currentSegmentation;
      tokenIt++;
    } // end for each token
  }

  // add explicit coverage edges for each node in the special annis namespace coverage component
  std::shared_ptr<WriteableGraphStorage> gsCoverage = db.createWritableGraphStorage(ComponentType::COVERAGE, annis_ns, "");
  std::shared_ptr<WriteableGraphStorage> gsInverseCoverage = db.createWritableGraphStorage(ComponentType::INVERSE_COVERAGE, annis_ns, "");
  HL_INFO(logger, "calculating the automatically generated COVERAGE edges");
  for(multimap<TextProperty, nodeid_t>::const_iterator itLeftToNode = leftToNode.begin();
      itLeftToNode != leftToNode.end(); itLeftToNode++)
  {
    nodeid_t n = itLeftToNode->second;

    TextProperty textPos;
    textPos.segmentation = "";
    textPos.textID = itLeftToNode->first.textID;
    textPos.corpusID = itLeftToNode->first.corpusID;

    uint32_t left = itLeftToNode->first.val;
    uint32_t right = nodeToRight[n];

    for(uint32_t i = left; i < right; i++)
    {
      // get the token that belongs to this text position
      textPos.val = i;
      nodeid_t tokenID = tokenByTextPosition[textPos];
      if(n != tokenID)
      {
        gsCoverage->addEdge(Init::initEdge(n, tokenID));
        gsInverseCoverage->addEdge(Init::initEdge(tokenID, n));
      }
    }
  }

  {
    string nodeAnnoTabPath = dirPath + "/node_annotation"  + (isANNIS33Format ? ".annis" : ".tab");
    HL_INFO(logger, (boost::format("loading %1%") % nodeAnnoTabPath).str());

    in.open(nodeAnnoTabPath, ifstream::in);
    if(!in.good()) return false;

    vector<string> line;
    while((line = Helper::nextCSV(in)).size() > 0)
    {
      // we have to make some sanity checks
      if(line[1] != "annis" || line[2] != "tok")
      {
        NodeAnnotationKey key;
        key.id = Helper::uint32FromString(line[0]);
        key.anno_ns = db.strings.add(line[1]);
        key.anno_name = db.strings.add(line[2]);

        uint32_t annoVal;
        if(line[3] == "NULL")
        {
          // use the empty string for empty annotations
          annoVal = db.strings.add("");
        }
        else
        {
          annoVal = db.strings.add(line[3]);
        }
        annoList.push_back({key, annoVal});

        // add all missing span values from the annotation, but don't add NULL values
        auto itMissing = missingSegmentationSpan.find(key.id);
        if(itMissing!= missingSegmentationSpan.end() && itMissing->second == line[2]
           && line[3] != "NULL")
        {
          Annotation tokAnno;
          tokAnno.ns = db.strings.add(annis_ns);
          tokAnno.name = db.strings.add(annis_tok);
          tokAnno.val = annoVal;
          annoList.push_back(std::pair<NodeAnnotationKey, uint32_t>({key.id, tokAnno.name, tokAnno.ns }, tokAnno.val));
        }
      }
    }

    in.close();
  }

  HL_INFO(logger, "bulk inserting node annotations");
  db.nodeAnnos.addAnnotationBulk(annoList);

  return true;
}


bool RelANNISLoader::loadRelANNISRank(const string &dirPath,
                          const map<uint32_t, std::shared_ptr<WriteableGraphStorage>>& componentToEdgeGS,
                          bool isANNIS33Format)
{
  typedef btree::btree_map<uint32_t, uint32_t>::const_iterator UintMapIt;
  typedef map<uint32_t, std::shared_ptr<WriteableGraphStorage>>::const_iterator ComponentIt;
  bool result = true;

  ifstream in;
  string rankTabPath = dirPath + "/rank" + (isANNIS33Format ? ".annis" : ".tab");
  HL_INFO(logger, (boost::format("loading %1%") % rankTabPath).str());

  in.open(rankTabPath, ifstream::in);
  if(!in.good()) return false;

  vector<string> line;

  const size_t nodeRefPos = isANNIS33Format ? 3 : 2;
  const size_t componentRefPos = isANNIS33Format ? 4 : 3;
  const size_t parentPos = isANNIS33Format ? 5 : 4;

  // first run: collect all pre-order values for a node
  btree::btree_map<uint32_t, uint32_t> pre2NodeID;
  map<uint32_t, Edge> pre2Edge;

  while((line = Helper::nextCSV(in)).size() > 0)
  {
    pre2NodeID.insert({Helper::uint32FromString(line[0]),Helper::uint32FromString(line[nodeRefPos])});
  }

  in.close();

  in.open(rankTabPath, ifstream::in);
  if(!in.good()) return false;

  map<uint32_t, std::shared_ptr<WriteableGraphStorage> > pre2GS;

  // second run: get the actual edges
  while((line = Helper::nextCSV(in)).size() > 0)
  {
    std::string parentAsString = line[parentPos];
    if(parentAsString != "NULL")
    {
      uint32_t parent = Helper::uint32FromString(parentAsString);
      UintMapIt it = pre2NodeID.find(parent);
      if(it != pre2NodeID.end())
      {
        // find the responsible edge database by the component ID
        ComponentIt itGS = componentToEdgeGS.find(Helper::uint32FromString(line[componentRefPos]));
        if(itGS != componentToEdgeGS.end())
        {
          std::shared_ptr<WriteableGraphStorage> gs = itGS->second;
          Edge edge = Init::initEdge(it->second, Helper::uint32FromString(line[nodeRefPos]));

          gs->addEdge(edge);
          pre2Edge[Helper::uint32FromString(line[0])] = edge;
          pre2GS[Helper::uint32FromString(line[0])] = gs;
        }
      }
      else
      {
        result = false;
      }
    }
  }
  in.close();


  if(result)
  {

    result = loadEdgeAnnotation(dirPath, pre2GS, pre2Edge, isANNIS33Format);
  }

  return result;
}


bool RelANNISLoader::loadEdgeAnnotation(const string &dirPath,
                            const map<uint32_t, std::shared_ptr<WriteableGraphStorage> >& pre2GS,
                            const map<uint32_t, Edge>& pre2Edge,
                            bool isANNIS33Format)
{

  bool result = true;

  ifstream in;
  string edgeAnnoTabPath = dirPath + "/edge_annotation" + (isANNIS33Format ? ".annis" : ".tab");
  HL_INFO(logger, (boost::format("loading %1%") % edgeAnnoTabPath).str());

  in.open(edgeAnnoTabPath, ifstream::in);
  if(!in.good()) return false;

  vector<string> line;

  while((line = Helper::nextCSV(in)).size() > 0)
  {
    uint32_t pre = Helper::uint32FromString(line[0]);
    map<uint32_t, std::shared_ptr<WriteableGraphStorage>>::const_iterator itDB = pre2GS.find(pre);
    map<uint32_t, Edge>::const_iterator itEdge = pre2Edge.find(pre);
    if(itDB != pre2GS.end() && itEdge != pre2Edge.end())
    {
      std::shared_ptr<WriteableGraphStorage> e = itDB->second;
      Annotation anno;
      anno.ns = db.strings.add(line[1]);
      anno.name = db.strings.add(line[2]);
      anno.val = db.strings.add(line[3]);
      if(e != NULL)
      {
        e->addEdgeAnnotation(itEdge->second, anno);
      }
    }
    else
    {
      result = false;
    }
  }

  in.close();

  return result;
}

void RelANNISLoader::loadCorpusAnnotation(const string &dirPath, std::multimap<uint32_t, Annotation>& corpusId2Annos,
                                          bool isANNIS33Format)
{

  ifstream in;
  string corpusAnnoTabPath = dirPath + "/corpus_annotation" + (isANNIS33Format ? ".annis" : ".tab");
  HL_INFO(logger, (boost::format("loading %1%") % corpusAnnoTabPath).str());

  in.open(corpusAnnoTabPath, ifstream::in);
  if(!in.good()) return;

  vector<string> line;

  while((line = Helper::nextCSV(in)).size() > 0)
  {
    std::string ns = "";
    if(line[1] != "NULL")
    {
      ns = line[1];
    }
    std::string name = line[2];
    std::string val = line[3];

    Annotation anno;
    anno.ns = db.strings.add(ns);
    anno.name = db.strings.add(name);
    anno.val = db.strings.add(val);

    corpusId2Annos.insert({Helper::uint32FromString(line[0]), anno});
  }

}

void RelANNISLoader::addSubCorpora(std::string toplevelCorpusName,
    std::map<uint32_t, uint32_t> &corpusByPreOrder, std::map<uint32_t, string> &corpusIDToName,
    multimap<uint32_t, nodeid_t>& nodesByCorpusID, std::multimap<uint32_t, Annotation>& corpusId2Annos)
{
  std::list<std::pair<NodeAnnotationKey, uint32_t>> corpusAnnoList;

  std::shared_ptr<WriteableGraphStorage> gsSubCorpus = db.createWritableGraphStorage(ComponentType::PART_OF_SUBCORPUS, annis_ns, "");

  nodeid_t nodeID = db.nextFreeNodeID();

  // add the toplevel corpus as node
  nodeid_t toplevelNodeID = nodeID++;
  corpusAnnoList.push_back({{toplevelNodeID, db.strings.add(annis_node_name), db.strings.add(annis_ns)},
                           db.strings.add(toplevelCorpusName)});
  {
    // add all metadata for the top-level corpus node
    auto itAnnoMeta = corpusId2Annos.equal_range(corpusByPreOrder[0]);
    for(auto it = itAnnoMeta.first; it != itAnnoMeta.second; it++)
    {
      corpusAnnoList.push_back({{toplevelNodeID, it->second.name, it->second.ns},
                                it->second.val});
    }
  }

  for(auto itCorpora = corpusByPreOrder.rbegin(); itCorpora != corpusByPreOrder.rend(); itCorpora++)
  {
    uint32_t corpusID = itCorpora->second;
    // add a node for the new (sub-) corpus/document
    std::string corpusName = corpusIDToName[corpusID];
    std::string fullName = toplevelCorpusName + "/" + corpusName;
    corpusAnnoList.push_back({{nodeID,  db.strings.add(annis_node_name), db.strings.add(annis_ns)},
                              db.strings.add(fullName)});
    corpusAnnoList.push_back({{nodeID,  db.strings.add("doc"), db.strings.add(annis_ns)},
                              db.strings.add(corpusName)});
    corpusAnnoList.push_back({{nodeID,  db.strings.add(annis_node_type), db.strings.add(annis_ns)},
                              db.strings.add("corpus")});

    // add all metadata for the document node
    auto itAnnoMeta = corpusId2Annos.equal_range(corpusID);
    for(auto it = itAnnoMeta.first; it != itAnnoMeta.second; it++)
    {
      corpusAnnoList.push_back({{nodeID, it->second.name, it->second.ns},
                                it->second.val});
    }

    // find all nodes belonging to this document and add a relation
    auto itNodeStart = nodesByCorpusID.lower_bound(corpusID);
    auto itNodeEnd  = nodesByCorpusID.upper_bound(corpusID);
    for(auto itNode = itNodeStart; itNode != itNodeEnd; itNode++)
    {
      gsSubCorpus->addEdge({nodeID, itNode->second});
    }

    // also add an edge from the top-level corpus to the document
    gsSubCorpus->addEdge({toplevelNodeID, nodeID});

    nodeID++;
  }

  db.nodeAnnos.addAnnotationBulk(corpusAnnoList);
}

ComponentType RelANNISLoader::componentTypeFromShortName(std::string shortType)
{
  ComponentType ctype;
  if(shortType == "c")
  {
    ctype = ComponentType::COVERAGE;
  }
  else if(shortType == "d")
  {
    ctype = ComponentType::DOMINANCE;
  }
  else if(shortType == "p")
  {
    ctype = ComponentType::POINTING;
  }
  else if(shortType == "o")
  {
    ctype = ComponentType::ORDERING;
  }
  else
  {
    throw("Unknown component type \"" + shortType + "\"");
  }
  return ctype;
}
