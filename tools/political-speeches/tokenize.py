#!/usr/bin/env python
# -*- coding: utf-8 -*-

import hashlib
from optparse import OptionParser

from lxml import etree
import somajo

def raw_sentence(tokens):
    tokensWithSpaces = []
    for token in tokens:
        tokensWithSpaces.append(str(token))
        if token.space_after:
            tokensWithSpaces.append(' ')
    return ''.join(tokensWithSpaces).strip()

if __name__ == "__main__":
    usage = "usage: %prog [options] corpus.xml corpus.conll"
    parser = OptionParser(usage)

    (options, args) = parser.parse_args()

    if len(args) != 2:
        parser.error("incorrect number of arguments")

    tokenizer = somajo.SoMaJo("de_CMC")

    with open(args[0]) as inputCorpus, open(args[1], "w") as outputCorpus:
        tree = etree.parse(inputCorpus)
        
        for text in tree.iter("text"):
            url = text.attrib["url"]

            title = text.attrib["titel"]
            subtitle = text.attrib["untertitel"]
            person = text.attrib["person"]
            date = text.attrib["datum"]
            # Use default value, 'ort' misses in various documents.
            location = text.attrib.get("ort", None)
            url = text.attrib["url"]
            salutation = text.attrib["anrede"]

            if not title:
                raise RuntimeError("Text without a title?")

            # URLs are to inconvenient
            if url:
                docId = "political-speeches-{}".format(hashlib.sha512(url.encode("utf-8")).hexdigest()[:10])
            else:
                docId = "political-speeches-{}".format(hashlib.sha512(title.encode("utf-8")).hexdigest()[:10])

            outputCorpus.write(f"# newdoc id = {docId}\n")
            if url:
                outputCorpus.write(f"# url = {url}\n");
            if title:
                outputCorpus.write(f"# title = {title}\n");
            if subtitle:
                outputCorpus.write(f"# subtitle = {subtitle}\n");
            if person:
                outputCorpus.write(f"# person = {person}\n");
            if date:
                outputCorpus.write(f"# date = {date}\n");
            if location:
                outputCorpus.write(f"# location = {location}\n");
            if salutation:
                outputCorpus.write(f"# salutation = {salutation}\n");

            # Text always has one child: rohtext
            rawText = text[0].text
            nPara = 0

            for para in rawText.splitlines():
                if not para.strip():
                    continue

                outputCorpus.write(f"# newpar id = {docId}-{nPara}\n")

                sents = tokenizer.tokenize_text([para])
                nSents = 0

                for sent in sents:
                    outputCorpus.write("# text = {}\n".format(raw_sentence(sent)))
                    outputCorpus.write(f"# sent_id = {docId}-{nPara}-{nSents}\n")
                    counter = 1
                    for token in sent:
                        outputCorpus.write("%d\t%s\n" % (counter, token))
                        counter += 1

                    outputCorpus.write("\n")
                    nSents += 1

                nPara += 1
