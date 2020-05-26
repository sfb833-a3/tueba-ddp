#!/usr/bin/env python
# -*- coding: utf-8 -*-

import re
import sys
import somajo

from optparse import OptionParser

idExpr = re.compile(r'id="([^"]+)".*title="([^"]+)"')

# Replace the tokens '†' and '*' by 'gestorben' and 'geboren' when they
# occur in parentheses. I first considered requiring that these are followed
# by dates. However, often, you will see things like (* Amsterdam). On the
# other hand, these tokens rarely occur in parentheses where it is not meant
# to be one of these words.
def replace_birth_death(sent):
    inParen = 0

    for idx in range(len(sent)):
        if sent[idx].text == '(':
            inParen += 1
        if sent[idx].text == ')':
            inParen -= 1

        if inParen > 0 and sent[idx].text == '†':
            sent[idx].text = 'gestorben'

        if inParen > 0 and sent[idx].text == '*':
            sent[idx] = 'geboren'

def raw_sentence(tokens):
    tokensWithSpaces = []
    for token in tokens:
        tokensWithSpaces.append(str(token))
        if token.space_after:
            tokensWithSpaces.append(' ')
    return ''.join(tokensWithSpaces).strip()


if __name__ == "__main__":
    usage = "usage: %prog [options]"
    parser = OptionParser(usage)
    parser.add_option("-w", "--wikipedia", action="store_true",
            dest="wikipedia", default=False, help="apply Wikipedia-specific substitutions")

    (options, args) = parser.parse_args()

    if len(args) != 0:
        parser.error("incorrect number of arguments")

    tokenizer = somajo.SoMaJo("de_CMC")

    doc_id = "err"
    title = "err"
    para = 0
    last_whitespace = True

    for line in sys.stdin:
        line = line.strip()

        if not line:
            if not last_whitespace:
                para += 1
            last_whitespace = True
            continue

        last_whitespace = False

        if line.startswith("<doc"):
            para = 0
            match = idExpr.search(line) 
            if match != None:
                doc_id = match.group(1)
                title = match.group(2)
            else:
                doc_id = "err"
                title = "err"

            print("# newdoc id = wiki-%s" % doc_id)
            print("# title = %s" % title)

            continue
        elif line.startswith("</doc"):
            continue
        

        sents = tokenizer.tokenize_text([line])

        print("# newpar id = wiki-%s-%d" % (doc_id, para))
        sent_id = 0
        for sent in sents:
            counter = 1
            print("# sent_id = wiki-%s-%d-%d" % (doc_id, para, sent_id))
            print("# text = %s" % raw_sentence(sent))

            if options.wikipedia:
                replace_birth_death(sent)

            for token in sent:
                print("%d\t%s" % (counter, token))
                counter += 1

            print()

            sent_id += 1

