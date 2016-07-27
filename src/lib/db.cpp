#include <annis/db.h>

#include <iostream>
#include <fstream>
#include <sstream>
#include <limits>

#if defined(__linux__) || defined(__linux) || defined(linux) || defined(__gnu_linux__)
#include <malloc.h>
#endif // LINUX

#include <boost/algorithm/string.hpp>
#include <boost/filesystem.hpp>
#include <boost/format.hpp>
#include <boost/archive/binary_iarchive.hpp>
#include <boost/archive/binary_oarchive.hpp>
#include <boost/serialization/set.hpp>

#include <humblelogging/api.h>

#include <annis/util/helper.h>
#include <annis/graphstorage/adjacencyliststorage.h>
#include <annis/graphstorage/linearstorage.h>
#include <annis/graphstorage/prepostorderstorage.h>
#include <annis/nodeannostorage.h>
#include <annis/graphstorage/graphstorage.h>
#include <annis/graphstorageregistry.h>

HUMBLE_LOGGER(logger, "annis4");

using namespace annis;
using namespace std;

DB::DB()
: nodeAnnos(strings), edges(strings)
{
  addDefaultStrings();
}

bool DB::load(string dirPath, bool preloadComponents)
{
  clear();
  addDefaultStrings();

  strings.load(dirPath);

  nodeAnnos.load(dirPath);

  edges.load(dirPath, preloadComponents);

  // TODO: return false on failure
  return true;
}

bool DB::save(string dirPath)
{

  boost::filesystem::create_directories(dirPath);

  strings.save(dirPath);
  nodeAnnos.save(dirPath);
  edges.save(dirPath);

  // TODO: return false on failure
  return true;
}

bool DB::loadRelANNIS(string dirPath)
{
  clear();
  addDefaultStrings();

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
  
  map<uint32_t, std::uint32_t> corpusIDToName;
  if(loadRelANNISCorpusTab(dirPath, corpusIDToName, isANNIS33Format) == false)
  {
    return false;
  }

  if(loadRelANNISNode(dirPath, corpusIDToName, isANNIS33Format) == false)
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
      ComponentType ctype = edges.componentTypeFromShortName(line[1]);
      std::shared_ptr<WriteableGraphStorage> gs = edges.createWritableGraphStorage(ctype, line[2], line[3]);
      componentToGS[componentID] = gs;
    }
  }

  in.close();

  bool result = loadRelANNISRank(dirPath, componentToGS, isANNIS33Format);


  // construct the complex indexes for all components
  std::list<Component> componentCopy;
  for(auto& gs : edges.container)
  {
    componentCopy.push_back(gs.first);
  }
  for(auto c : componentCopy)
  {
    convertComponent(c);
  }

  HL_INFO(logger, "Updating statistics");
  nodeAnnos.calculateStatistics();

  #if defined(__linux__) || defined(__linux) || defined(linux) || defined(__gnu_linux__)
  malloc_trim(0);
  #endif // LINUX

  HL_INFO(logger, "Finished loading relANNIS");
  return result;
}


bool DB::loadRelANNISCorpusTab(string dirPath, map<uint32_t, std::uint32_t>& corpusIDToName,
  bool isANNIS33Format)
{
  string corpusTabPath = dirPath + "/corpus" + (isANNIS33Format ? ".annis" : ".tab");
  HL_INFO(logger, (boost::format("loading %1%") % corpusTabPath).str());

  ifstream in;
  in.open(corpusTabPath, ifstream::in);
  if(!in.good())
  {
    string msg = "Can't find corpus";
    msg += (isANNIS33Format ? ".annis" : ".tab");
    HL_ERROR(logger, msg);
    return false;
  }
  vector<string> line;
  while((line = Helper::nextCSV(in)).size() > 0)
  {
    std::uint32_t corpusID = Helper::uint32FromString(line[0]);
    std::uint32_t nameID = strings.add(line[1]);
    corpusIDToName[corpusID] = nameID;
  }
  return true;
}

bool DB::loadRelANNISNode(string dirPath, map<uint32_t, std::uint32_t>& corpusIDToName,
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

      bool hasSegmentations = line.size() > 10;
      string tokenIndexRaw = line[7];
      uint32_t textID = Helper::uint32FromString(line[1]);
      uint32_t corpusID = Helper::uint32FromString(line[2]);

      Annotation nodeNameAnno;
      nodeNameAnno.ns = strings.add(annis_ns);
      nodeNameAnno.name = strings.add(annis_node_name);
      nodeNameAnno.val = strings.add(line[4]);
      annoList.push_back(std::pair<NodeAnnotationKey, uint32_t>({nodeNr, nodeNameAnno.name, nodeNameAnno.ns }, nodeNameAnno.val));


      Annotation documentNameAnno;
      documentNameAnno.ns = strings.add(annis_ns);
      documentNameAnno.name = strings.add("document");
      documentNameAnno.val = corpusIDToName[corpusID];
      annoList.push_back(std::pair<NodeAnnotationKey, uint32_t>({nodeNr, documentNameAnno.name, documentNameAnno.ns }, documentNameAnno.val));

      TextProperty left;
      left.val = Helper::uint32FromString(line[5]);
      left.textID = textID;

      TextProperty right;
      right.val = Helper::uint32FromString(line[6]);
      right.textID = textID;

      if(tokenIndexRaw != "NULL")
      {
        string span = hasSegmentations ? line[12] : line[9];

        Annotation tokAnno;
        tokAnno.ns = strings.add(annis_ns);
        tokAnno.name = strings.add(annis_tok);
        tokAnno.val = strings.add(span);
        annoList.push_back(std::pair<NodeAnnotationKey, uint32_t>({nodeNr, tokAnno.name, tokAnno.ns }, tokAnno.val));

        TextProperty index;
        index.val = Helper::uint32FromString(tokenIndexRaw);
        index.textID = textID;

        tokenByIndex[index] = nodeNr;

        TextProperty textPos;
        textPos.textID = textID;
        for(uint32_t i=left.val; i <= right.val; i++)
        {
          textPos.val = i;
          tokenByTextPosition[textPos] = nodeNr;
        }

      } // end if token

      leftToNode.insert(pair<TextProperty, uint32_t>(left, nodeNr));
      rightToNode.insert(pair<TextProperty, uint32_t>(right, nodeNr));
      nodeToLeft[nodeNr] = left.val;
      nodeToRight[nodeNr] = right.val;
    }

    in.close();
  }

  // TODO: cleanup, better variable naming and put this into it's own function
  // iterate over all token by their order, find the nodes with the same
  // text coverage (either left or right) and add explicit ORDERING, LEFT_TOKEN and RIGHT_TOKEN edges
  if(!tokenByIndex.empty())
  {
    HL_INFO(logger, "calculating the automatically generated ORDERING, LEFT_TOKEN and RIGHT_TOKEN edges");
    std::shared_ptr<WriteableGraphStorage> gsOrder = edges.createWritableGraphStorage(ComponentType::ORDERING, annis_ns, "");
    std::shared_ptr<WriteableGraphStorage> gsLeft = edges.createWritableGraphStorage(ComponentType::LEFT_TOKEN, annis_ns, "");
    std::shared_ptr<WriteableGraphStorage> gsRight = edges.createWritableGraphStorage(ComponentType::RIGHT_TOKEN, annis_ns, "");

    map<TextProperty, uint32_t>::const_iterator tokenIt = tokenByIndex.begin();
    uint32_t lastTextID = numeric_limits<uint32_t>::max();
    uint32_t lastToken = numeric_limits<uint32_t>::max();

    while(tokenIt != tokenByIndex.end())
    {
      uint32_t currentToken = tokenIt->second;
      uint32_t currentTextID = tokenIt->first.textID;

      // find all nodes that start together with the current token
      TextProperty currentTokenLeft;
      currentTokenLeft.textID = currentTextID;
      currentTokenLeft.val = nodeToLeft[currentToken];
      pair<TextPropIt, TextPropIt> leftAlignedNodes = leftToNode.equal_range(currentTokenLeft);
      for(TextPropIt itLeftAligned=leftAlignedNodes.first; itLeftAligned != leftAlignedNodes.second; itLeftAligned++)
      {
        gsLeft->addEdge(Init::initEdge(itLeftAligned->second, currentToken));
        gsLeft->addEdge(Init::initEdge(currentToken, itLeftAligned->second));
      }

      // find all nodes that end together with the current token
      TextProperty currentTokenRight;
      currentTokenRight.textID = currentTextID;
      currentTokenRight.val = nodeToRight[currentToken];
      pair<TextPropIt, TextPropIt> rightAlignedNodes = rightToNode.equal_range(currentTokenRight);
      for(TextPropIt itRightAligned=rightAlignedNodes.first; itRightAligned != rightAlignedNodes.second; itRightAligned++)
      {
        gsRight->addEdge(Init::initEdge(itRightAligned->second, currentToken));
        gsRight->addEdge(Init::initEdge(currentToken, itRightAligned->second));
      }

      // if the last token/text value is valid and we are still in the same text
      if(tokenIt != tokenByIndex.begin() && currentTextID == lastTextID)
      {
        // we are still in the same text
        uint32_t nextToken = tokenIt->second;
        // add ordering between token
        gsOrder->addEdge(Init::initEdge(lastToken, nextToken));

      } // end if same text

      // update the iterator and other variables
      lastTextID = currentTextID;
      lastToken = tokenIt->second;
      tokenIt++;
    } // end for each token
  }

  // add explicit coverage edges for each node in the special annis namespace coverage component
  std::shared_ptr<WriteableGraphStorage> gsCoverage = edges.createWritableGraphStorage(ComponentType::COVERAGE, annis_ns, "");
  std::shared_ptr<WriteableGraphStorage> gsInverseCoverage = edges.createWritableGraphStorage(ComponentType::INVERSE_COVERAGE, annis_ns, "");
  HL_INFO(logger, "calculating the automatically generated COVERAGE edges");
  for(multimap<TextProperty, nodeid_t>::const_iterator itLeftToNode = leftToNode.begin();
      itLeftToNode != leftToNode.end(); itLeftToNode++)
  {
    nodeid_t n = itLeftToNode->second;

    TextProperty textPos;
    textPos.textID = itLeftToNode->first.textID;

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
      NodeAnnotationKey key;
      key.node = Helper::uint32FromString(line[0]);
      key.anno_ns = strings.add(line[1]);
      key.anno_name = strings.add(line[2]);

      uint32_t annoVal = strings.add(line[3]);
      annoList.push_back({key, annoVal});
    }

    in.close();
  }

  HL_INFO(logger, "bulk inserting node annotations");
  nodeAnnos.addNodeAnnotationBulk(annoList);

  return true;
}


bool DB::loadRelANNISRank(const string &dirPath,
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


bool DB::loadEdgeAnnotation(const string &dirPath,
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
      anno.ns = strings.add(line[1]);
      anno.name = strings.add(line[2]);
      anno.val = strings.add(line[3]);
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

void DB::clear()
{
  strings.clear();
  nodeAnnos.clear();
  edges.clear();
}

void DB::addDefaultStrings()
{
  annisNamespaceStringID = strings.add(annis_ns);
  annisEmptyStringID = strings.add("");
  annisTokStringID = strings.add(annis_tok);
  annisNodeNameStringID = strings.add(annis_node_name);
}

void DB::convertComponent(Component c, std::string impl)
{
  map<Component, std::shared_ptr<ReadableGraphStorage>>::const_iterator
      it = edges.container.find(c);
  if(it != edges.container.end())
  {
    std::shared_ptr<ReadableGraphStorage> oldStorage = it->second;

    if(!(oldStorage->getStatistics().valid))
    {
      oldStorage->calculateStatistics();
    }

    std::string currentImpl = edges.registry.getName(oldStorage);
    if(impl == "")
    {
      impl = edges.registry.getOptimizedImpl(c, oldStorage->getStatistics());
    }
    std::shared_ptr<ReadableGraphStorage> newStorage = oldStorage;
    if(currentImpl != impl)
    {
      HL_DEBUG(logger, (boost::format("converting component %1% from %2% to %3%")
                       % edges.debugComponentString(c)
                       % currentImpl
                       % impl).str());

      newStorage = edges.registry.createGraphStorage(impl, strings, c);
      newStorage->copy(*this, *oldStorage);
      edges.container[c] = newStorage;
    }

    // perform index calculations
    std::shared_ptr<WriteableGraphStorage> asWriteableGS = std::dynamic_pointer_cast<WriteableGraphStorage>(newStorage);
    if(asWriteableGS)
    {
      asWriteableGS->calculateIndex();
    }
  }
}

void DB::optimizeAll(const std::map<Component, string>& manualExceptions)
{
  for(const auto& c : getAllComponents())
  {
    edges.ensureComponentIsLoaded(c);
    auto find = manualExceptions.find(c);
    if(find == manualExceptions.end())
    {
      // get the automatic calculated best implementation
      convertComponent(c);
    }
    else
    {
      convertComponent(c, find->second);
    }
  }
}

void DB::ensureAllComponentsLoaded()
{
  for(const auto& c : getAllComponents())
  {
    edges.ensureComponentIsLoaded(c);
  }
}

size_t DB::estimateMemorySize()
{
  return
    nodeAnnos.estimateMemorySize()
      + strings.estimateMemorySize()
      + edges.estimateMemorySize();
}

string DB::info()
{
  stringstream ss;
  ss  << "Number of node annotations: " << nodeAnnos.nodeAnnotations.size() << endl
      << "Number of strings in storage: " << strings.size() << endl
      << "Average string length: " << strings.avgLength() << endl
      << "--------------------" << std::endl
      << edges.info() << std::endl;

  return ss.str();
}


std::vector<Component> DB::getDirectConnected(const Edge &edge) const
{
  std::vector<Component> result;
  map<Component, std::shared_ptr<ReadableGraphStorage>>::const_iterator itGS = edges.container.begin();

  while(itGS != edges.container.end())
  {
    std::shared_ptr<ReadableGraphStorage> gs = itGS->second;
    if(gs != NULL)
    {
      if(gs->isConnected(edge))
      {
        result.push_back(itGS->first);
      }
    }
    itGS++;
  }

  return result;
}

std::vector<Component> DB::getAllComponents() const
{
  std::vector<Component> result;
  map<Component, std::shared_ptr<ReadableGraphStorage>>::const_iterator itGS = edges.container.begin();

  while(itGS != edges.container.end())
  {
    result.push_back(itGS->first);
    itGS++;
  }

  return result;
}

vector<Annotation> DB::getEdgeAnnotations(const Component &component,
                                          const Edge &edge)
{
  std::map<Component,std::shared_ptr<ReadableGraphStorage>>::const_iterator it = edges.container.find(component);
  if(it != edges.container.end() && it->second != NULL)
  {
    std::shared_ptr<ReadableGraphStorage> gs = it->second;
    return gs->getEdgeAnnotations(edge);
  }

  return vector<Annotation>();

}

DB::~DB()
{
}

