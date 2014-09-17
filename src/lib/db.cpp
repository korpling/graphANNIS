#include "db.h"

#include <iostream>
#include <fstream>
#include <sstream>
#include <limits>

#include <boost/algorithm/string.hpp>
#include <boost/filesystem.hpp>
#include <boost/format.hpp>
#include <humblelogging/api.h>

#include "edgedb/fallbackedgedb.h"

HUMBLE_LOGGER(logger, "annis4");

using namespace annis;
using namespace std;

DB::DB()
{
}

vector<string> DB::nextCSV(istream& in)
{
  vector<string> result;
  string line;

  getline(in, line);
  stringstream lineStream(line);
  string cell;

  while(getline(lineStream, cell, '\t'))
  {
    boost::replace_all(cell, "\\\\", "\\");
    boost::replace_all(cell, "\\t", "\t");
    boost::replace_all(cell, "\\n", "\n");
    result.push_back(cell);
  }
  return result;
}

void DB::writeCSVLine(ostream &out, vector<string> data)
{

  vector<string>::const_iterator it = data.begin();
  while(it != data.end())
  {
    string s = *it;
    boost::replace_all(s, "\t", "\\t");
    boost::replace_all(s, "\n", "\\n");
    boost::replace_all(s, "\\", "\\\\");

    out << s;
    it++;
    if(it != data.end())
    {
      out << "\t";
    }
  }
}



bool DB::load(string dirPath)
{
  clear();

  ifstream in;
/*
  in.open(dirPath + "/edges.btree");
  edges.restore(in);
  in.close();
*/
  in.open(dirPath + "/nodeAnnotations.btree");
  nodeAnnotations.restore(in);
  in.close();

  /*
  in.open(dirPath + "/edgeAnnotations.btree");
  edgeAnnotations.restore(in);
  in.close();
*/

  // load the strings from CSV
  in.open(dirPath + "/strings.list");
  vector<string> line;
  while((line = nextCSV(in)).size() > 0)
  {
    uint32_t id = uint32FromString(line[0]);
    stringStorageByID.insert2(id, line[1]);
    stringStorageByValue.insert2(line[1], id);
  }
  in.close();

  // TODO: return false on failure
  return true;
}

bool DB::save(string dirPath)
{
  typedef stx::btree<uint32_t, string>::const_iterator StringStorageIt;

  boost::filesystem::create_directories(dirPath);

  ofstream out;

/*
  out.open(dirPath + "/edges.btree");
  edges.dump(out);
  out.close();
*/
  out.open(dirPath + "/nodeAnnotations.btree");
  nodeAnnotations.dump(out);
  out.close();

/*
  out.open(dirPath + "/edgeAnnotations.btree");
  edgeAnnotations.dump(out);
  out.close();
*/

  // load the strings from CSV
  out.open(dirPath + "/strings.list");
  StringStorageIt it = stringStorageByID.begin();
  while(it != stringStorageByID.end())
  {
    vector<string> line;
    line.push_back(stringFromUInt32(it->first));
    line.push_back(it->second);
    writeCSVLine(out, line);
    it++;
    if(it != stringStorageByID.end())
    {
      out << "\n";
    }
  }
  out.close();

  // TODO: return false on failure
  return true;
}

bool DB::loadRelANNIS(string dirPath)
{
  clear();


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
  while((line = nextCSV(in)).size() > 0)
  {
    uint32_t nodeNr;
    stringstream nodeNrStream(line[0]);
    nodeNrStream >> nodeNr;

    bool hasSegmentations = line.size() > 10;
    string token_index = line[7];
    string span = hasSegmentations ? line[12] : line[9];

    if(token_index == "NULL")
    {
      // add at least one dummy annotation so we know the node is there
      // TODO: remove all these later if other annotations are found
      Annotation nodeAnno;
      nodeAnno.ns = addString(annis_ns);
      nodeAnno.name = addString("node");
      nodeAnno.val = addString("");
      nodeAnnotations.insert2(nodeNr, nodeAnno);
    }
    else
    {
      Annotation tokAnno;
      tokAnno.ns = addString(annis_ns);
      tokAnno.name = addString("tok");
      tokAnno.val = addString(span);
      nodeAnnotations.insert2(nodeNr, tokAnno);
    }
  }

  in.close();

  string nodeAnnoTabPath = dirPath + "/node_annotation.tab";
  HL_INFO(logger, (boost::format("loading %1%") % nodeAnnoTabPath).str());

  in.open(nodeAnnoTabPath, ifstream::in);
  if(!in.good()) return false;

  while((line = nextCSV(in)).size() > 0)
  {
    u_int32_t nodeNr = uint32FromString(line[0]);
    Annotation anno;
    anno.ns = addString(line[1]);
    anno.name = addString(line[2]);
    anno.val = addString(line[3]);
    nodeAnnotations.insert2(nodeNr, anno);
  }

  in.close();

  string componentTabPath = dirPath + "/component.tab";
  HL_INFO(logger, (boost::format("loading %1%") % componentTabPath).str());

  in.open(componentTabPath, ifstream::in);
  if(!in.good()) return false;

  map<uint32_t, EdgeDB*> componentToEdgeDB;
  while((line = nextCSV(in)).size() > 0)
  {
    uint32_t componentID = uint32FromString(line[0]);
    EdgeDB* edb = createEdgeDBForComponent(line[1], line[2], line[3]);
    componentToEdgeDB[componentID] = edb;
  }

  in.close();

  bool result = loadRelANNISRank(dirPath, componentToEdgeDB);


  return result;
}

bool DB::loadRelANNISRank(const string &dirPath,
                          map<uint32_t, EdgeDB*>& componentToEdgeDB)
{
  typedef stx::btree_map<uint32_t, uint32_t>::const_iterator UintMapIt;

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

  while((line = nextCSV(in)).size() > 0)
  {
    pre2NodeID.insert2(uint32FromString(line[0]),uint32FromString(line[2]));
  }

  in.close();

  in.open(rankTabPath, ifstream::in);
  if(!in.good()) return false;

  map<uint32_t, EdgeDB* > pre2EdgeDB;

  // second run: get the actual edges
  while((line = nextCSV(in)).size() > 0)
  {
    uint32_t parent = uint32FromString(line[4]);
    UintMapIt it = pre2NodeID.find(parent);
    if(it != pre2NodeID.end())
    {
      // find the responsible edge database by the component ID
      EdgeDB* edb = componentToEdgeDB[uint32FromString(line[3])];
      pair<uint32_t, uint32_t> edge(uint32FromString(line[2]), it->second);

      edb->addEdge(edge);
      pre2Edge[uint32FromString(line[0])] = edge;
      pre2EdgeDB[uint32FromString(line[0])] = edb;
    }
    else
    {
      result = false;
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

  while((line = nextCSV(in)).size() > 0)
  {
    uint32_t pre = uint32FromString(line[0]);
    map<uint32_t, EdgeDB*>::const_iterator itDB = pre2EdgeDB.find(pre);
    map<uint32_t, Edge>::const_iterator itEdge = pre2Edge.find(pre);
    if(itDB != pre2EdgeDB.end() && itEdge != pre2Edge.end())
    {
      EdgeDB* e = itDB->second;
      Annotation anno;
      anno.ns = addString(line[1]);
      anno.name = addString(line[2]);
      anno.val = addString(line[3]);
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

uint32_t DB::addString(const string &str)
{
  typedef stx::btree_map<string, uint32_t>::const_iterator ItType;
  ItType it = stringStorageByValue.find(str);
  if(it == stringStorageByValue.end())
  {
    // non-existing
    uint32_t id = 0;
    if(stringStorageByID.size() > 0)
    {
      id = ((stringStorageByID.rbegin())->first)+1;
    }
    stringStorageByID.insert2(id, str);
    stringStorageByValue.insert2(str, id);
    return id;
  }
  else
  {
    // already existing, return the original ID
    return it->second;
  }
}

void DB::clear()
{
  nodeAnnotations.clear();
  stringStorageByID.clear();
  stringStorageByValue.clear();
}

EdgeDB *DB::createEdgeDBForComponent(const string &type, const string &ns, const string &name)
{
  // fill the component variable
  Component c;
  if(type == "c")
  {
    c.type = ComponentType::COVERAGE;
  }
  else if(type == "d")
  {
    c.type = ComponentType::DOMINANCE;
  }
  else if(type == "p")
  {
    c.type = ComponentType::POINTING;
  }
  else if(type == "o")
  {
    c.type = ComponentType::ORDERING;
  }
  else
  {
    throw("Unknown component type \"" + type + "\"");
  }
  if(ns.size() < MAX_COMPONENT_NAME_SIZE-1 && name.size() < MAX_COMPONENT_NAME_SIZE-1)
  {
    memset(c.ns, 0, MAX_COMPONENT_NAME_SIZE);
    memset(c.name, 0, MAX_COMPONENT_NAME_SIZE);
    ns.copy(c.ns, ns.size());
    if(name != "NULL")
    {
      name.copy(c.name, name.size());
    }
  }
  else
  {
    throw("Component name or namespace are too long");
  }


  // check if there is already an edge DB for this component
  map<Component,EdgeDB*,compComponent>::const_iterator itDB =
      edgeDatabases.find(c);
  if(itDB == edgeDatabases.end())
  {

    // TODO: decide which implementation to use
    EdgeDB* edgeDB = new FallbackEdgeDB(c);

    // register the used implementation
    edgeDatabases.insert(pair<Component,EdgeDB*>(c,edgeDB));
    return edgeDB;
  }
  else
  {
    return itDB->second;
  }
}

bool DB::hasNode(uint32_t id)
{
  stx::btree_multimap<uint32_t, Annotation>::const_iterator itNode = nodeAnnotations.find(id);
  if(itNode == nodeAnnotations.end())
  {
    return false;
  }
  else
  {
    return true;
  }
}

/*
vector<Edge> DB::getInEdges(uint32_t nodeID)
{
  typedef stx::btree_set<Edge, compEdges>::const_iterator UintSetIt;
  vector<Edge> result;
  Edge keyLower;
  Edge keyUpper;

  keyLower.target = nodeID;
  keyUpper.target = nodeID;

  keyLower.source = numeric_limits<uint32_t>::min();
  keyLower.component = numeric_limits<uint32_t>::min();

  keyUpper.source = numeric_limits<uint32_t>::max();
  keyUpper.component = numeric_limits<uint32_t>::max();

  UintSetIt lowerBound = edges.lower_bound(keyLower);
  UintSetIt upperBound = edges.upper_bound(keyUpper);

  for(UintSetIt it=lowerBound; it != upperBound; it++)
  {
    result.push_back(*it);
  }

  return result;
}
*/

string DB::info()
{
  stringstream ss;
  ss  << "Number of node annotations: " << nodeAnnotations.size() << endl
      << "Number of strings in storage: " << stringStorageByID.size();
  return ss.str();
}

/*
vector<Edge> DB::getEdgesBetweenNodes(uint32_t sourceID, uint32_t targetID)
{
  typedef stx::btree_set<Edge, compEdges>::const_iterator UintSetIt;
  vector<Edge> result;
  Edge keyLower;
  Edge keyUpper;

  keyLower.source = sourceID;
  keyUpper.source = sourceID;
  keyLower.target = targetID;
  keyUpper.target = targetID;

  keyLower.component = numeric_limits<uint32_t>::min();
  keyUpper.component = numeric_limits<uint32_t>::max();

  UintSetIt lowerBound = edges.lower_bound(keyLower);
  UintSetIt upperBound = edges.upper_bound(keyUpper);

  for(UintSetIt it=lowerBound; it != upperBound; it++)
  {
    result.push_back(*it);
  }

  return result;
}
*/

vector<Annotation> DB::getNodeAnnotationsByID(const uint32_t& id)
{
  typedef stx::btree_multimap<uint32_t, Annotation>::const_iterator AnnoIt;

  vector<Annotation> result;

  pair<AnnoIt,AnnoIt> itRange = nodeAnnotations.equal_range(id);

  for(AnnoIt itAnnos = itRange.first;
      itAnnos != itRange.second; itAnnos++)
  {
    result.push_back(itAnnos->second);
  }

  return result;
}

std::vector<Component> DB::getDirectConnected(const Edge &edge)
{
  std::vector<Component> result;
  map<Component, EdgeDB*>::const_iterator itEdgeDB = edgeDatabases.begin();

  while(itEdgeDB != edgeDatabases.end())
  {
    EdgeDB* edb = itEdgeDB->second;
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

vector<Annotation> DB::getEdgeAnnotations(const Component &component,
                                          const Edge &edge)
{
  std::map<Component,EdgeDB*>::const_iterator it = edgeDatabases.find(component);
  if(it != edgeDatabases.end() && it->second != NULL)
  {
    EdgeDB* edb = it->second;
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
