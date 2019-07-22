#!/usr/bin/env python
# -*- coding: utf-8 -*-

from lxml import etree
import somajo

from optparse import OptionParser

if __name__ == "__main__":
    usage = "usage: %prog [options] corpus.xml corpus.conll sent_doc_ids"
    parser = OptionParser(usage)

    (options, args) = parser.parse_args()

    if len(args) != 3:
        parser.error("incorrect number of arguments")

    tokenizer = somajo.Tokenizer()
    splitter = somajo.SentenceSplitter(is_tuple=False)

    with open(args[0]) as inputCorpus, open(args[1], "w") as outputCorpus, open(args[2], "w") as sentDocIds:
        tree = etree.parse(inputCorpus)
        
        for text in tree.iter("text"):
            url = text.attrib["url"]

            anrede = text.attrib["anrede"]

            # Text always has one child: rohtext
            rawText = text[0].text

            tokens = tokenizer.tokenize(rawText)
            sents = splitter.split(tokens)

            for sent in sents:
                counter = 1
                for token in sent:
                    outputCorpus.write("%d\t%s\n" % (counter, token))
                    counter += 1

                outputCorpus.write("\n")

                sentDocIds.write("%s\n" % url)
