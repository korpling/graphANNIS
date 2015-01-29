#include "db.h"

#include <iostream>
#include <fstream>
#include <sstream>
#include <limits>

#include <boost/algorithm/string.hpp>
#include <boost/filesystem.hpp>
#include <boost/format.hpp>
#include <humblelogging/api.h>

#include "helper.h"
#include "edgedb/fallbackedgedb.h"
#include "edgedb/linearedgedb.h"
#include "edgedb/coverageedb.h"
#include "edgedb/prepostorderstorage.h"

HUMBLE_LOGGER(logger, "annis4");

using namespace annis;
using namespace std;

DB::DB(bool useSpecializedEdgeDB)
  : useSpecializedEdgeDB(useSpecializedEdgeDB)
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

  boost::filesystem::directory_iterator fileEndIt;

  for(unsigned int componentType = (unsigned int) ComponentType::COVERAGE;
      componentType < (unsigned int) ComponentType::ComponentType_MAX; componentType++)
  {
    const boost::filesystem::path componentPath(dirPath + "/edgedb/"
                                                + ComponentTypeHelper::toString((ComponentType) componentType));

    if(boost::filesystem::is_directory(componentPath))
    {
      // get all the namespaces/layers
      boost::filesystem::directory_iterator itLayers(componentPath);
      while(itLayers != fileEndIt)
      {
        const boost::filesystem::path layerPath = *itLayers;

        std::string implName = getImplNameForPath(layerPath.string());

        // try to load the component with the empty name
        Component emptyNameComponent = {(ComponentType) componentType,
            layerPath.filename().string(), ""};
        ReadableGraphStorage* edbEmptyName = registry.createEdgeDB(implName, strings, emptyNameComponent);
        edbEmptyName->load(layerPath.string());
        edgeDatabases.insert(std::pair<Component,ReadableGraphStorage*>(emptyNameComponent,edbEmptyName));

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
            ReadableGraphStorage* edbNamed = registry.createEdgeDB(implName, strings, namedComponent);
            edbNamed->load(namedComponentPath.string());
            edgeDatabases.insert(std::pair<Component,ReadableGraphStorage*>(namedComponent,edbNamed));
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

  // save each edge db separately
  string edgeDBParent = dirPath + "/edgedb";
  for(EdgeDBIt it = edgeDatabases.begin(); it != edgeDatabases.end(); it++)
  {
    const Component& c = it->first;
    string finalPath;
    if(c.name.empty())
    {
      finalPath = edgeDBParent + "/" + ComponentTypeHelper::toString(c.type) + "/" + c.layer;
    }
    else
    {
      finalPath = edgeDBParent + "/" + ComponentTypeHelper::toString(c.type) + "/" + c.layer + "/" + c.name;
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

  map<uint32_t, EdgeDB*> componentToEdgeDB;
  while((line = Helper::nextCSV(in)).size() > 0)
  {
    uint32_t componentID = Helper::uint32FromString(line[0]);
    if(line[1] != "NULL")
    {
      ComponentType ctype = componentTypeFromShortName(line[1]);
      EdgeDB* edb = createWritableEdgeDB(ctype, line[2], line[3]);
      componentToEdgeDB[componentID] = edb;
    }
  }

  in.close();

  bool result = loadRelANNISRank(dirPath, componentToEdgeDB);


  // construct the complex indexes for all components
  std::list<Component> componentCopy;
  for(auto& ed : edgeDatabases)
  {
    componentCopy.push_back(ed.first);
  }
  for(auto c : componentCopy)
  {
    HL_INFO(logger, (boost::format("component calculations %1%|%2%|%3%")
                     % ComponentTypeHelper::toString(c.type)
                     % c.layer
                     % c.name).str());
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
    EdgeDB* edbOrder = createWritableEdgeDB(ComponentType::ORDERING, annis_ns, "");
    EdgeDB* edbLeft = createWritableEdgeDB(ComponentType::LEFT_TOKEN, annis_ns, "");
    EdgeDB* edbRight = createWritableEdgeDB(ComponentType::RIGHT_TOKEN, annis_ns, "");

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
        edbLeft->addEdge(Init::initEdge(itLeftAligned->second, currentToken));
        edbLeft->addEdge(Init::initEdge(currentToken, itLeftAligned->second));
      }

      // find all nodes that end together with the current token
      TextProperty currentTokenRight;
      currentTokenRight.textID = currentTextID;
      currentTokenRight.val = nodeToRight[currentToken];
      pair<TextPropIt, TextPropIt> rightAlignedNodes = rightToNode.equal_range(currentTokenRight);
      for(TextPropIt itRightAligned=rightAlignedNodes.first; itRightAligned != rightAlignedNodes.second; itRightAligned++)
      {
        edbRight->addEdge(Init::initEdge(itRightAligned->second, currentToken));
        edbRight->addEdge(Init::initEdge(currentToken, itRightAligned->second));
      }

      // if the last token/text value is valid and we are still in the same text
      if(tokenIt != tokenByIndex.begin() && currentTextID == lastTextID)
      {
        // we are still in the same text
        uint32_t nextToken = tokenIt->second;
        // add ordering between token
        edbOrder->addEdge(Init::initEdge(lastToken, nextToken));

      } // end if same text

      // update the iterator and other variables
      lastTextID = currentTextID;
      lastToken = tokenIt->second;
      tokenIt++;
    } // end for each token
  }

  // add explicit coverage edges for each node in the special annis namespace coverage component
  EdgeDB* edbCoverage = createWritableEdgeDB(ComponentType::COVERAGE, annis_ns, "");
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
        edbCoverage->addEdge(Init::initEdge(n, tokenID));
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
                          const map<uint32_t, EdgeDB*>& componentToEdgeDB)
{
  typedef stx::btree_map<uint32_t, uint32_t>::const_iterator UintMapIt;
  typedef map<uint32_t, EdgeDB*>::const_iterator ComponentIt;
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

  map<uint32_t, EdgeDB* > pre2EdgeDB;

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
        ComponentIt itEdb = componentToEdgeDB.find(Helper::uint32FromString(line[3]));
        if(itEdb != componentToEdgeDB.end())
        {
          EdgeDB* edb = itEdb->second;
          Edge edge = Init::initEdge(it->second, Helper::uint32FromString(line[2]));

          edb->addEdge(edge);
          pre2Edge[Helper::uint32FromString(line[0])] = edge;
          pre2EdgeDB[Helper::uint32FromString(line[0])] = edb;
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

    result = loadEdgeAnnotation(dirPath, pre2EdgeDB, pre2Edge);
  }

  return result;
}


bool DB::loadEdgeAnnotation(const string &dirPath,
                            const map<uint32_t, EdgeDB* >& pre2EdgeDB,
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
    map<uint32_t, EdgeDB*>::const_iterator itDB = pre2EdgeDB.find(pre);
    map<uint32_t, Edge>::const_iterator itEdge = pre2Edge.find(pre);
    if(itDB != pre2EdgeDB.end() && itEdge != pre2Edge.end())
    {
      EdgeDB* e = itDB->second;
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

ReadableGraphStorage *DB::createEdgeDBForComponent(const string &shortType, const string &layer, const string &name)
{
  // fill the component variable
  ComponentType ctype = componentTypeFromShortName(shortType);
  return createEdgeDBForComponent(ctype, layer, name);

}

ReadableGraphStorage *DB::createEdgeDBForComponent(ComponentType ctype, const string &layer, const string &name)
{
  Component c = {ctype, layer, name};

  // check if there is already an edge DB for this component
  map<Component,ReadableGraphStorage*>::const_iterator itDB =
      edgeDatabases.find(c);
  if(itDB == edgeDatabases.end())
  {

    // TODO: decide which implementation to use
    ReadableGraphStorage* edgeDB = NULL;
    if(useSpecializedEdgeDB)
    {
      edgeDB = registry.createEdgeDB(strings, c);
    }
    else
    {
      edgeDB = registry.createEdgeDB(registry.fallback, strings, c);
    }

    // register the used implementation
    edgeDatabases.insert(pair<Component,ReadableGraphStorage*>(c,edgeDB));
    return edgeDB;
  }
  else
  {
    return itDB->second;
  }
}

EdgeDB* DB::createWritableEdgeDB(ComponentType ctype, const string &layer, const string &name)
{
  Component c = {ctype, layer, name == "NULL" ? "" : name};

  // check if there is already an edge DB for this component
  map<Component,ReadableGraphStorage*>::const_iterator itDB =
      edgeDatabases.find(c);
  if(itDB != edgeDatabases.end())
  {
    // check if the current implementation is writeable
    EdgeDB* writable = dynamic_cast<EdgeDB*>(itDB->second);
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

  EdgeDB* edgeDB = new FallbackEdgeDB(strings, c);
  // register the used implementation
  edgeDatabases.insert(pair<Component,ReadableGraphStorage*>(c,edgeDB));
  return edgeDB;

}

void DB::convertComponent(Component c, std::string optimizedImpl)
{
  map<Component, ReadableGraphStorage*>::const_iterator
      it = edgeDatabases.find(c);
  if(it != edgeDatabases.end())
  {
    ReadableGraphStorage* oldStorage = it->second;

    std::string currentImpl = registry.getName(oldStorage);
    if(optimizedImpl == "")
    {
      optimizedImpl = registry.getOptimizedImpl(c);
    }
    ReadableGraphStorage* newStorage = oldStorage;
    if(currentImpl != optimizedImpl)
    {

//      std::cerr << "converting component " << ComponentTypeHelper::toString(c.type)
//                << " " << c.layer << ":" << c.name << " from " << currentImpl << " to " << optimizedImpl << std::endl;
      newStorage = registry.createEdgeDB(optimizedImpl, strings, c);
      newStorage->copy(*this, *oldStorage);
      edgeDatabases[c] = newStorage;
      delete oldStorage;
    }

    // perform index calculations
    EdgeDB* asEdgeDB = dynamic_cast<EdgeDB*>(newStorage);
    if(asEdgeDB != nullptr)
    {
      asEdgeDB->calculateIndex();
    }
  }
}

string DB::getImplNameForPath(string directory)
{
  std::string result = registry.fallback;
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

bool DB::hasNode(nodeid_t id)
{
  stx::btree_multimap<nodeid_t, Annotation>::const_iterator itNode = nodeAnnotations.find(id);
  if(itNode == nodeAnnotations.end())
  {
    return false;
  }
  else
  {
    return true;
  }
}

string DB::info()
{
  stringstream ss;
  ss  << "Number of node annotations: " << nodeAnnotations.size() << endl
      << "Number of strings in storage: " << strings.size() << endl;

  for(EdgeDBIt it = edgeDatabases.begin(); it != edgeDatabases.end(); it++)
  {
    const Component& c = it->first;
    const ReadableGraphStorage* edb = it->second;
    ss << "Component " << ComponentTypeHelper::toString(c.type) << "|" << c.layer
       << "|" << c.name << ": " << edb->numberOfEdges() << " edges and "
       << edb->numberOfEdgeAnnotations() << " annotations" << endl;
  }

  return ss.str();
}


std::vector<Component> DB::getDirectConnected(const Edge &edge) const
{
  std::vector<Component> result;
  map<Component, ReadableGraphStorage*>::const_iterator itEdgeDB = edgeDatabases.begin();

  while(itEdgeDB != edgeDatabases.end())
  {
    ReadableGraphStorage* edb = itEdgeDB->second;
    if(edb != NULL)
    {
      if(edb->isConnected(edge))
      {
        result.push_back(itEdgeDB->first);
      }
    }
    itEdgeDB++;
  }

  return result;
}

std::vector<Component> DB::getAllComponents() const
{
  std::vector<Component> result;
  map<Component, ReadableGraphStorage*>::const_iterator itEdgeDB = edgeDatabases.begin();

  while(itEdgeDB != edgeDatabases.end())
  {
    result.push_back(itEdgeDB->first);
    itEdgeDB++;
  }

  return result;
}

const ReadableGraphStorage* DB::getEdgeDB(const Component &component) const
{
  map<Component, ReadableGraphStorage*>::const_iterator itEdgeDB = edgeDatabases.find(component);
  if(itEdgeDB != edgeDatabases.end())
  {
    return itEdgeDB->second;
  }
  return NULL;
}

const ReadableGraphStorage *DB::getEdgeDB(ComponentType type, const string &layer, const string &name) const
{
  Component c = {type, layer, name};
  return getEdgeDB(c);
}

std::vector<const ReadableGraphStorage *> DB::getEdgeDB(ComponentType type, const string &name) const
{
  std::vector<const ReadableGraphStorage* > result;

  Component componentKey;
  componentKey.type = type;
  componentKey.layer[0] = '\0';
  componentKey.name[0] = '\0';

  for(auto itEdgeDB = edgeDatabases.lower_bound(componentKey);
      itEdgeDB != edgeDatabases.end() && itEdgeDB->first.type == type;
      itEdgeDB++)
  {
    const Component& c = itEdgeDB->first;
    if(name == c.name)
    {
      result.push_back(itEdgeDB->second);
    }
  }

  return result;
}

std::vector<const ReadableGraphStorage *> DB::getAllEdgeDBForType(ComponentType type) const
{
  std::vector<const ReadableGraphStorage* > result;

  Component c;
  c.type = type;
  c.layer[0] = '\0';
  c.name[0] = '\0';

  for(
      map<Component, ReadableGraphStorage*>::const_iterator itEdgeDB = edgeDatabases.lower_bound(c);
      itEdgeDB != edgeDatabases.end() && itEdgeDB->first.type == type;
      itEdgeDB++)
  {
    result.push_back(itEdgeDB->second);
  }

  return result;
}

vector<Annotation> DB::getEdgeAnnotations(const Component &component,
                                          const Edge &edge)
{
  std::map<Component,ReadableGraphStorage*>::const_iterator it = edgeDatabases.find(component);
  if(it != edgeDatabases.end() && it->second != NULL)
  {
    ReadableGraphStorage* edb = it->second;
    return edb->getEdgeAnnotations(edge);
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

