#ifndef HELPER_H
#define HELPER_H

#include <istream>
#include <ostream>
#include <vector>
#include <string>
#include <boost/algorithm/string.hpp>
#include <sstream>

namespace annis
{

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
  std::stringstream lineStream(line);
  std::string cell;

  while(std::getline(lineStream, cell, '\t'))
  {
    boost::replace_all(cell, "\\\\", "\\");
    boost::replace_all(cell, "\\t", "\t");
    boost::replace_all(cell, "\\n", "\n");
    result.push_back(cell);
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
} // end namespace annis

#endif // HELPER_H
