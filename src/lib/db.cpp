#include "db.h"

#include <iostream>
#include <fstream>
#include <sstream>
#include <limits>

#include <boost/algorithm/string.hpp>
#include <boost/filesystem.hpp>
#include <boost/format.hpp>
#include <boost/archive/binary_iarchive.hpp>
#include <boost/archive/binary_oarchive.hpp>
#include <boost/serialization/set.hpp>

#include <humblelogging/api.h>

#include "helper.h"
#include "graphstorage/adjacencyliststorage.h"
#include "graphstorage/linearstorage.h"
#include "graphstorage/prepostorderstorage.h"

HUMBLE_LOGGER(logger, "annis4");

using namespace annis;
using namespace std;

DB::DB()
{
  addDefaultStrings();
}

bool DB::load(string dirPath)
{
  clear();
  addDefaultStrings();

  HL_INFO(logger, "Start loading string storage");
  strings.load(dirPath);
  HL_INFO(logger, "End loading string storage");

  ifstream in;

  in.open(dirPath + "/nodeAnnotations.btree");
  nodeAnnotations.restore(in);
  in.close();

  in.open(dirPath + "/inverseNodeAnnotations.btree");
  inverseNodeAnnotations.restore(in);
  in.close();

  in.open(dirPath + "/nodeAnnoKeys.archive");
  boost::archive::binary_iarchive iaNodeAnnoKeys(in);
  iaNodeAnnoKeys >> nodeAnnoKeys;
  in.close();

  boost::filesystem::directory_iterator fileEndIt;

  for(unsigned int componentType = (unsigned int) ComponentType::COVERAGE;
      componentType < (unsigned int) ComponentType::ComponentType_MAX; componentType++)
  {
    const boost::filesystem::path componentPath(dirPath + "/gs/"
                                                + ComponentTypeHelper::toString((ComponentType) componentType));

    if(boost::filesystem::is_directory(componentPath))
    {
      // get all the namespaces/layers
      boost::filesystem::directory_iterator itLayers(componentPath);
      while(itLayers != fileEndIt)
      {
        const boost::filesystem::path layerPath = *itLayers;

        std::string implName = getImplNameForPath(layerPath.string());

        if(!implName.empty())
        {
          // try to load the component with the empty name
          Component emptyNameComponent = {(ComponentType) componentType,
              layerPath.filename().string(), ""};
          HL_INFO(logger, (boost::format("loading component %1%")
                           % debugComponentString(emptyNameComponent)).str());

          ReadableGraphStorage* gsEmptyName = registry.createGraphStorage(implName, strings, emptyNameComponent);
          gsEmptyName->load(layerPath.string());
          edgeDatabases.insert(std::pair<Component,ReadableGraphStorage*>(emptyNameComponent,gsEmptyName));
        }

        // also load all named components
        boost::filesystem::directory_iterator itNamedComponents(layerPath);
        while(itNamedComponents != fileEndIt)
        {
          const boost::filesystem::path namedComponentPath = *itNamedComponents;
          if(boost::filesystem::is_directory(namedComponentPath))
          {
            // try to load the named component
            implName = getImplNameForPath(namedComponentPath.string());
            Component namedComponent = {(ComponentType) componentType,
                                                           layerPath.filename().string(),
                                                           namedComponentPath.filename().string()
                                       };
            HL_INFO(logger, (boost::format("loading component %1%")
                             % debugComponentString(namedComponent)).str());
            ReadableGraphStorage* gsNamed = registry.createGraphStorage(implName, strings, namedComponent);
            gsNamed->load(namedComponentPath.string());
            edgeDatabases.insert(std::pair<Component,ReadableGraphStorage*>(namedComponent,gsNamed));
          }
          itNamedComponents++;
        } // end for each file/directory in layer directory
        itLayers++;
      } // for each layers
    }
  } // end for each component

  // TODO: return false on failure
  HL_INFO(logger, "Finished loading");
  return true;
}

bool DB::save(string dirPath)
{

  boost::filesystem::create_directories(dirPath);

  strings.save(dirPath);

  ofstream out;

  out.open(dirPath + "/nodeAnnotations.btree");
  nodeAnnotations.dump(out);
  out.close();

  out.open(dirPath + "/inverseNodeAnnotations.btree");
  inverseNodeAnnotations.dump(out);
  out.close();

  out.open(dirPath + "/nodeAnnoKeys.archive");
  boost::archive::binary_oarchive oaNodeAnnoKeys(out);
  oaNodeAnnoKeys << nodeAnnoKeys;
  out.close();

  // save each edge db separately
  string gsParent = dirPath + "/gs";
  for(GraphStorageIt it = edgeDatabases.begin(); it != edgeDatabases.end(); it++)
  {
    const Component& c = it->first;
    string finalPath;
    if(c.name.empty())
    {
      finalPath = gsParent + "/" + ComponentTypeHelper::toString(c.type) + "/" + c.layer;
    }
    else
    {
      finalPath = gsParent + "/" + ComponentTypeHelper::toString(c.type) + "/" + c.layer + "/" + c.name;
    }
    boost::filesystem::create_directories(finalPath);
    it->second->save(finalPath);
    // put an identification file to the output directory that contains the name of the graph storage implementation
    out.open(finalPath + "/implementation.cfg");
    out << registry.getName(it->second) << std::endl;
    out.close();
  }


  // TODO: return false on failure
  return true;
}

bool DB::loadRelANNIS(string dirPath)
{
  clear();
  addDefaultStrings();

  map<uint32_t, std::uint32_t> corpusIDToName;
  if(loadRelANNISCorpusTab(dirPath, corpusIDToName) == false)
  {
    return false;
  }

  if(loadRelANNISNode(dirPath, corpusIDToName) == false)
  {
    return false;
  }

  string componentTabPath = dirPath + "/component.tab";
  HL_INFO(logger, (boost::format("loading %1%") % componentTabPath).str());

  ifstream in;
  vector<string> line;

  in.open(componentTabPath, ifstream::in);
  if(!in.good()) return false;

  map<uint32_t, WriteableGraphStorage*> componentToGS;
  while((line = Helper::nextCSV(in)).size() > 0)
  {
    uint32_t componentID = Helper::uint32FromString(line[0]);
    if(line[1] != "NULL")
    {
      ComponentType ctype = componentTypeFromShortName(line[1]);
      WriteableGraphStorage* gs = createWritableGraphStorage(ctype, line[2], line[3]);
      componentToGS[componentID] = gs;
    }
  }

  in.close();

  bool result = loadRelANNISRank(dirPath, componentToGS);


  // construct the complex indexes for all components
  std::list<Component> componentCopy;
  for(auto& ed : edgeDatabases)
  {
    componentCopy.push_back(ed.first);
  }
  for(auto c : componentCopy)
  {
    convertComponent(c);
  }
  HL_INFO(logger, "Finished loading relANNIS");
  return result;
}


bool DB::loadRelANNISCorpusTab(string dirPath, map<uint32_t, std::uint32_t>& corpusIDToName)
{
  string corpusTabPath = dirPath + "/corpus.tab";
  HL_INFO(logger, (boost::format("loading %1%") % corpusTabPath).str());

  ifstream in;
  in.open(corpusTabPath, ifstream::in);
  if(!in.good())
  {
    HL_ERROR(logger, "Can't find corpus.tab");
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

bool DB::loadRelANNISNode(string dirPath, map<uint32_t, std::uint32_t>& corpusIDToName)
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

  string nodeTabPath = dirPath + "/node.tab";
  HL_INFO(logger, (boost::format("loading %1%") % nodeTabPath).str());

  ifstream in;
  in.open(nodeTabPath, ifstream::in);
  if(!in.good())
  {
    HL_ERROR(logger, "Can't find node.tab");
    return false;
  }
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
    addNodeAnnotation(nodeNr, nodeNameAnno);

    Annotation documentNameAnno;
    documentNameAnno.ns = strings.add(annis_ns);
    documentNameAnno.name = strings.add("document");
    documentNameAnno.val = corpusIDToName[corpusID];
    addNodeAnnotation(nodeNr, documentNameAnno);

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
      addNodeAnnotation(nodeNr, tokAnno);

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

  // TODO: cleanup, better variable naming and put this into it's own function
  // iterate over all token by their order, find the nodes with the same
  // text coverage (either left or right) and add explicit ORDERING, LEFT_TOKEN and RIGHT_TOKEN edges
  if(!tokenByIndex.empty())
  {
    HL_INFO(logger, "calculating the automatically generated ORDERING, LEFT_TOKEN and RIGHT_TOKEN edges");
    WriteableGraphStorage* gsOrder = createWritableGraphStorage(ComponentType::ORDERING, annis_ns, "");
    WriteableGraphStorage* gsLeft = createWritableGraphStorage(ComponentType::LEFT_TOKEN, annis_ns, "");
    WriteableGraphStorage* gsRight = createWritableGraphStorage(ComponentType::RIGHT_TOKEN, annis_ns, "");

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
  WriteableGraphStorage* gsCoverage = createWritableGraphStorage(ComponentType::COVERAGE, annis_ns, "");
  WriteableGraphStorage* gsInverseCoverage = createWritableGraphStorage(ComponentType::INVERSE_COVERAGE, annis_ns, "");
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

  string nodeAnnoTabPath = dirPath + "/node_annotation.tab";
  HL_INFO(logger, (boost::format("loading %1%") % nodeAnnoTabPath).str());

  in.open(nodeAnnoTabPath, ifstream::in);
  if(!in.good()) return false;

  while((line = Helper::nextCSV(in)).size() > 0)
  {
    u_int32_t nodeNr = Helper::uint32FromString(line[0]);
    Annotation anno;
    anno.ns = strings.add(line[1]);
    anno.name = strings.add(line[2]);
    anno.val = strings.add(line[3]);
    addNodeAnnotation(nodeNr, anno);
  }

  in.close();
  return true;
}


bool DB::loadRelANNISRank(const string &dirPath,
                          const map<uint32_t, WriteableGraphStorage*>& componentToEdgeGS)
{
  typedef stx::btree_map<uint32_t, uint32_t>::const_iterator UintMapIt;
  typedef map<uint32_t, WriteableGraphStorage*>::const_iterator ComponentIt;
  bool result = true;

  ifstream in;
  string rankTabPath = dirPath + "/rank.tab";
  HL_INFO(logger, (boost::format("loading %1%") % rankTabPath).str());

  in.open(rankTabPath, ifstream::in);
  if(!in.good()) return false;

  vector<string> line;

  // first run: collect all pre-order values for a node
  stx::btree_map<uint32_t, uint32_t> pre2NodeID;
  map<uint32_t, Edge> pre2Edge;

  while((line = Helper::nextCSV(in)).size() > 0)
  {
    pre2NodeID.insert2(Helper::uint32FromString(line[0]),Helper::uint32FromString(line[2]));
  }

  in.close();

  in.open(rankTabPath, ifstream::in);
  if(!in.good()) return false;

  map<uint32_t, WriteableGraphStorage* > pre2GS;

  // second run: get the actual edges
  while((line = Helper::nextCSV(in)).size() > 0)
  {
    std::string parentAsString = line[4];
    if(parentAsString != "NULL")
    {
      uint32_t parent = Helper::uint32FromString(parentAsString);
      UintMapIt it = pre2NodeID.find(parent);
      if(it != pre2NodeID.end())
      {
        // find the responsible edge database by the component ID
        ComponentIt itGS = componentToEdgeGS.find(Helper::uint32FromString(line[3]));
        if(itGS != componentToEdgeGS.end())
        {
          WriteableGraphStorage* gs = itGS->second;
          Edge edge = Init::initEdge(it->second, Helper::uint32FromString(line[2]));

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

    result = loadEdgeAnnotation(dirPath, pre2GS, pre2Edge);
  }

  return result;
}


bool DB::loadEdgeAnnotation(const string &dirPath,
                            const map<uint32_t, WriteableGraphStorage* >& pre2GS,
                            const map<uint32_t, Edge>& pre2Edge)
{

  bool result = true;

  ifstream in;
  string edgeAnnoTabPath = dirPath + "/edge_annotation.tab";
  HL_INFO(logger, (boost::format("loading %1%") % edgeAnnoTabPath).str());

  in.open(edgeAnnoTabPath, ifstream::in);
  if(!in.good()) return false;

  vector<string> line;

  while((line = Helper::nextCSV(in)).size() > 0)
  {
    uint32_t pre = Helper::uint32FromString(line[0]);
    map<uint32_t, WriteableGraphStorage*>::const_iterator itDB = pre2GS.find(pre);
    map<uint32_t, Edge>::const_iterator itEdge = pre2Edge.find(pre);
    if(itDB != pre2GS.end() && itEdge != pre2Edge.end())
    {
      WriteableGraphStorage* e = itDB->second;
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
  nodeAnnotations.clear();
  inverseNodeAnnotations.clear();
  for(auto& ed : edgeDatabases)
  {
    delete ed.second;
  }
  edgeDatabases.clear();
}

void DB::addDefaultStrings()
{
  annisNamespaceStringID = strings.add(annis_ns);
  annisEmptyStringID = strings.add("");
  annisTokStringID = strings.add(annis_tok);
  annisNodeNameStringID = strings.add(annis_node_name);
}

ReadableGraphStorage *DB::createGSForComponent(const string &shortType, const string &layer, const string &name)
{
  // fill the component variable
  ComponentType ctype = componentTypeFromShortName(shortType);
  return createGSForComponent(ctype, layer, name);

}

ReadableGraphStorage *DB::createGSForComponent(ComponentType ctype, const string &layer, const string &name)
{
  Component c = {ctype, layer, name};

  // check if there is already an edge DB for this component
  map<Component,ReadableGraphStorage*>::const_iterator itDB =
      edgeDatabases.find(c);
  if(itDB == edgeDatabases.end())
  {

    ReadableGraphStorage* gs = NULL;
    gs = registry.createGraphStorage(strings, c, gs->getStatistics());

    // register the used implementation
    edgeDatabases.insert(pair<Component,ReadableGraphStorage*>(c,gs));
    return gs;
  }
  else
  {
    return itDB->second;
  }
}

WriteableGraphStorage* DB::createWritableGraphStorage(ComponentType ctype, const string &layer, const string &name)
{
  Component c = {ctype, layer, name == "NULL" ? "" : name};

  // check if there is already an edge DB for this component
  map<Component,ReadableGraphStorage*>::const_iterator itDB =
      edgeDatabases.find(c);
  if(itDB != edgeDatabases.end())
  {
    // check if the current implementation is writeable
    WriteableGraphStorage* writable = dynamic_cast<WriteableGraphStorage*>(itDB->second);
    if(writable != nullptr)
    {
      return writable;
    }
    else
    {
      ReadableGraphStorage* old = itDB->second;
      edgeDatabases.erase(itDB);
      delete old;
    }
  }

  WriteableGraphStorage* gs = new AdjacencyListStorage(strings, c);
  // register the used implementation
  edgeDatabases.insert(pair<Component,ReadableGraphStorage*>(c,gs));
  return gs;

}

void DB::convertComponent(Component c, std::string impl)
{
  map<Component, ReadableGraphStorage*>::const_iterator
      it = edgeDatabases.find(c);
  if(it != edgeDatabases.end())
  {
    ReadableGraphStorage* oldStorage = it->second;

    if(!(oldStorage->getStatistics().valid))
    {
      oldStorage->calculateStatistics();
    }

    std::string currentImpl = registry.getName(oldStorage);
    if(impl == "")
    {
      impl = registry.getOptimizedImpl(c, oldStorage->getStatistics());
    }
    ReadableGraphStorage* newStorage = oldStorage;
    if(currentImpl != impl)
    {
      HL_INFO(logger, (boost::format("converting component %1% from %2% to %3%")
                       % debugComponentString(c)
                       % currentImpl
                       % impl).str());

      newStorage = registry.createGraphStorage(impl, strings, c);
      newStorage->copy(*this, *oldStorage);
      edgeDatabases[c] = newStorage;
      delete oldStorage;
    }

    // perform index calculations
    WriteableGraphStorage* asWriteableGS = dynamic_cast<WriteableGraphStorage*>(newStorage);
    if(asWriteableGS != nullptr)
    {
      asWriteableGS->calculateIndex();
    }
  }
}

void DB::optimizeAll(const std::map<Component, string>& manualExceptions)
{
  for(const auto& c : getAllComponents())
  {
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

string DB::getImplNameForPath(string directory)
{
  std::string result = "";
  std::ifstream in(directory + "/implementation.cfg");
  if(in.is_open())
  {
    in >> result;
  }
  in.close();
  return result;
}

ComponentType DB::componentTypeFromShortName(string shortType)
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

string DB::info()
{
  stringstream ss;
  ss  << "Number of node annotations: " << nodeAnnotations.size() << endl
      << "Number of strings in storage: " << strings.size() << endl
      << "Average string length: " << strings.avgLength() << endl;

  for(GraphStorageIt it = edgeDatabases.begin(); it != edgeDatabases.end(); it++)
  {
    const Component& c = it->first;
    const ReadableGraphStorage* gs = it->second;


    ss << "Component " << debugComponentString(c) << ": " << gs->numberOfEdges() << " edges and "
       << gs->numberOfEdgeAnnotations() << " annotations" << endl;


    std::string implName = registry.getName(gs);
    if(!implName.empty())
    {
      ss << "implementation: " << implName << endl;
    }

    GraphStatistic stat = gs->getStatistics();
    if(stat.valid)
    {
      ss << "nodes: " << stat.nodes << endl;
      ss << "fan-out: " << stat.avgFanOut << " (avg) / " << stat.maxFanOut << " (max)" << endl;
      if(stat.cyclic)
      {
        ss << "cyclic" << endl;
      }
      else
      {
        ss << "non-cyclic, max. depth: " << stat.maxDepth << ", DFS visit ratio: " << stat.dfsVisitRatio << endl;

      }
      if(stat.rootedTree)
      {
        ss << "rooted tree" << endl;
      }
    }
    ss << "--------------------" << endl;
  }

  return ss.str();
}


std::vector<Component> DB::getDirectConnected(const Edge &edge) const
{
  std::vector<Component> result;
  map<Component, ReadableGraphStorage*>::const_iterator itGS = edgeDatabases.begin();

  while(itGS != edgeDatabases.end())
  {
    ReadableGraphStorage* gs = itGS->second;
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
  map<Component, ReadableGraphStorage*>::const_iterator itGS = edgeDatabases.begin();

  while(itGS != edgeDatabases.end())
  {
    result.push_back(itGS->first);
    itGS++;
  }

  return result;
}

const ReadableGraphStorage* DB::getGraphStorage(const Component &component) const
{
  map<Component, ReadableGraphStorage*>::const_iterator itGS = edgeDatabases.find(component);
  if(itGS != edgeDatabases.end())
  {
    return itGS->second;
  }
  return NULL;
}

const ReadableGraphStorage *DB::getGraphStorage(ComponentType type, const string &layer, const string &name) const
{
  Component c = {type, layer, name};
  return getGraphStorage(c);
}

std::vector<const ReadableGraphStorage *> DB::getGraphStorage(ComponentType type, const string &name) const
{
  std::vector<const ReadableGraphStorage* > result;

  Component componentKey;
  componentKey.type = type;
  componentKey.layer[0] = '\0';
  componentKey.name[0] = '\0';

  for(auto itGS = edgeDatabases.lower_bound(componentKey);
      itGS != edgeDatabases.end() && itGS->first.type == type;
      itGS++)
  {
    const Component& c = itGS->first;
    if(name == c.name)
    {
      result.push_back(itGS->second);
    }
  }

  return result;
}

std::vector<const ReadableGraphStorage *> DB::getGraphStorage(ComponentType type) const
{
  std::vector<const ReadableGraphStorage* > result;

  Component c;
  c.type = type;
  c.layer[0] = '\0';
  c.name[0] = '\0';

  for(
      map<Component, ReadableGraphStorage*>::const_iterator itGS = edgeDatabases.lower_bound(c);
      itGS != edgeDatabases.end() && itGS->first.type == type;
      itGS++)
  {
    result.push_back(itGS->second);
  }

  return result;
}

vector<Annotation> DB::getEdgeAnnotations(const Component &component,
                                          const Edge &edge)
{
  std::map<Component,ReadableGraphStorage*>::const_iterator it = edgeDatabases.find(component);
  if(it != edgeDatabases.end() && it->second != NULL)
  {
    ReadableGraphStorage* gs = it->second;
    return gs->getEdgeAnnotations(edge);
  }

  return vector<Annotation>();

}

DB::~DB()
{
  for(auto& ed : edgeDatabases)
  {
    delete ed.second;
  }
}

