#pragma once

#include <istream>
#include <ostream>
#include <vector>
#include <string>
#include <boost/algorithm/string.hpp>
#include <sstream>

#include <annis/db.h>
#include <annis/graphstorage/graphstorage.h>
#include <annis/graphstorageholder.h>

#ifdef WIN32
#include <windows.h>
#else
#include <sys/time.h>
#include <cstdlib>
#include <ctime>
#endif

namespace annis
{

static const unsigned long long long_thousand = 1000;

class TokenHelper
{
public:

  TokenHelper(GraphStorageHolder& gsh, const DB& db) : db(db),
    leftEdges(gsh.getGraphStorage(ComponentType::LEFT_TOKEN, annis_ns, "")),
    rightEdges(gsh.getGraphStorage(ComponentType::RIGHT_TOKEN, annis_ns, ""))
  {

  }
  
  std::pair<nodeid_t, nodeid_t> leftRightTokenForNode(const nodeid_t& n)
  {
    if(isToken(n))
    {
      return {n, n};
    }
    else
    {
      return {leftEdges->getOutgoingEdges(n)[0],  rightEdges->getOutgoingEdges(n)[0]};
    }
  }

  nodeid_t leftTokenForNode(const nodeid_t& n)
  {
    if(isToken(n))
    {
      return n;
    }
    else
    {
      return leftEdges->getOutgoingEdges(n)[0];
    }
  }

  nodeid_t rightTokenForNode(const nodeid_t& n)
  {
    if(isToken(n))
    {
      return n;
    }
    else
    {
      return rightEdges->getOutgoingEdges(n)[0];
    }
  }

  bool inline isToken(const nodeid_t& n)
  {
    return db.nodeAnnos.getAnnotation(n, db.getNamespaceStringID(), db.getTokStringID()).first;
  }

private:
  const DB& db;
  std::shared_ptr<const ReadableGraphStorage> leftEdges;
  std::shared_ptr<const ReadableGraphStorage> rightEdges;
};

class Helper
{
public:
  static std::uint32_t uint32FromString(const std::string& str)
  {
    std::uint32_t result = 0;
    std::stringstream stream(str);
    stream >> result;
    return result;
  }

  static std::string stringFromUInt32(const std::uint32_t& val)
  {
    std::stringstream stream("");
    stream << val;
    return stream.str();
  }

  static std::vector<std::string> nextCSV(std::istream &in)
  {
    std::vector<std::string> result;
    std::string line;

    std::getline(in, line);
    if(!line.empty())
    {
      boost::split(result, line, boost::is_any_of("\t"), boost::token_compress_off);

      for(auto& cell : result)
      {
        boost::replace_all(cell, "\\t", "\t");
        boost::replace_all(cell, "\\'", "'");
        boost::replace_all(cell, "\\\\", "\\");
      }
    }
    return result;
  }

  static void writeCSVLine(std::ostream &out, std::vector<std::string> data)
  {
    std::vector<std::string>::const_iterator it = data.begin();
    while(it != data.end())
    {
      std::string s = *it;
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

  static unsigned long long getSystemTimeInMilliSeconds()
  {
#ifdef WIN32
    LARGE_INTEGER highPerformanceTick;
    LARGE_INTEGER freq;
    if(QueryPerformanceCounter(&highPerformanceTick) && QueryPerformanceFrequency(&freq)) {
      double inSeconds = ((double) highPerformanceTick.LowPart) / ((double) freq.LowPart);
      return (unsigned long long) (inSeconds * 1000.0);
    } else {
      return 0;
    }
#else
    struct timeval t;
    int returnval = gettimeofday(&t, NULL);
    if(returnval == 0) {
      return ((unsigned long long)t.tv_sec) * long_thousand + ((unsigned long long)t.tv_usec) / long_thousand;
    } else {
      return 0;
    }
#endif
  }//end getSystemTimeInMilliSeconds

  static double inMB(size_t inBytes)
  {
    return (double) inBytes / (double) 1048576.0;
  }
};



} // end namespace annis
