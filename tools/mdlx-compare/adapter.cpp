#include <whiteout/models/mdx/parser.h>
#include <whiteout/models/mdx/writer.h>
#include <fstream>
#include <iostream>
#include <stdexcept>
#include <string>

using whiteout::mdx::Parser;
using whiteout::mdx::Writer;
using whiteout::mdx::MdlFormat;

using std::cerr;
using std::exception;
using std::ifstream;
using std::ios;
using std::runtime_error;
using std::string;

// INPUT OUTPUT DIALECT; the input extension selects MDX or MDL.
int main(int argc, char** argv) {
    try {
        if (argc != 4) throw runtime_error("usage: whiteout-adapter INPUT OUTPUT engine|hive");
        const string dialect = argv[3];
        if (dialect != "engine" && dialect != "hive") throw runtime_error("invalid dialect");
        Parser parser(Parser::UpgradeMode::PreserveOriginal);
        const auto model = parser.parse(argv[1]);
        for (const auto& issue : parser.getIssues()) cerr << issue << '\n';
        Writer writer;
        writer.write(argv[2], model, dialect == "hive" ? MdlFormat::Hiveworkshop : MdlFormat::WarcraftIII);
        ifstream output(argv[2], ios::binary | ios::ate);
        if (!output || output.tellg() <= 0) throw runtime_error("writer produced no output");
        return 0;
    } catch (const exception& error) {
        cerr << error.what() << '\n';
        return 1;
    }
}
