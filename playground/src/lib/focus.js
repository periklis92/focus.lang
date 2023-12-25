ace.define("ace/mode/focus_highlight_rules", ["require", "exports", "module", "ace/lib/oop", "ace/mode/text_highlight_rules"], function (acequire, exports, module) {
    "use strict";

    var oop = acequire("../lib/oop");
    var TextHighlightRules = acequire("./text_highlight_rules").TextHighlightRules;

    var FocusHighlightRules = function () {

        var keywords = (
            "let|fn|and|not|is|as|or|and|match|if|then|else|from|import"
        );

        var builtinConstants = ("true|false");

        var keywordMapper = this.createKeywordMapper({
            "variable.language": "this",
            "keyword": keywords,
            "constant.language": builtinConstants,
        }, "identifier");

        var decimalInteger = "(?:(?:[1-9]\\d*)|(?:0))";
        var octInteger = "(?:0[oO]?[0-7]+)";
        var hexInteger = "(?:0[xX][\\dA-Fa-f]+)";
        var binInteger = "(?:0[bB][01]+)";
        var integer = "(?:" + decimalInteger + "|" + octInteger + "|" + hexInteger + "|" + binInteger + ")";

        var exponent = "(?:[eE][+-]?\\d+)";
        var fraction = "(?:\\.\\d+)";
        var intPart = "(?:\\d+)";
        var pointFloat = "(?:(?:" + intPart + "?" + fraction + ")|(?:" + intPart + "\\.))";
        var exponentFloat = "(?:(?:" + pointFloat + "|" + intPart + ")" + exponent + ")";
        var floatNumber = "(?:" + exponentFloat + "|" + pointFloat + ")";

        this.$rules = {
            "start": [
                {
                    token: "comment",
                    regex: '#.*'
                },
                {
                    token: "entity.other",
                    regex: "[A-Z][a-z]*"
                },
                {
                    token: "support.function",
                    regex: "(?<=\\.)\\w+"
                },
                {
                    token: "string", // single line
                    regex: '["](?:(?:\\\\.)|(?:[^"\\\\]))*?["]'
                },
                {
                    token: "string", // single char
                    regex: "'.'"
                },
                {
                    token: "string", // " string
                    regex: '"',
                    next: "qstring"
                },
                {
                    token: "constant.numeric", // imaginary
                    regex: "(?:" + floatNumber + "|\\d+)[jJ]\\b"
                },
                {
                    token: "constant.numeric", // float
                    regex: floatNumber
                },
                {
                    token: "constant.numeric", // integer
                    regex: integer + "\\b"
                },
                {
                    token: keywordMapper,
                    regex: "[a-zA-Z_$][a-zA-Z0-9_$]*\\b"
                },
                {
                    token: "keyword.operator",
                    regex: "\\+\\.|\\-\\.|\\*\\.|\\/\\.|#|;;|\\+|\\-|\\*|\\*\\*\\/|\\/\\/|%|<<|>>|&|\\||\\^|~|<|>|<=|=>|==|!=|<>|<-|="
                },
                {
                    token: "paren.lparen",
                    regex: "[[({]"
                },
                {
                    token: "paren.rparen",
                    regex: "[\\])}]"
                },
                {
                    token: "text",
                    regex: "\\s+"
                }
            ],
            "comment": [
                {
                    token: "comment", // closing comment
                    regex: "\\*\\)",
                    next: "start"
                },
                {
                    defaultToken: "comment"
                }
            ],

            "qstring": [
                {
                    token: "string",
                    regex: '"',
                    next: "start"
                }, {
                    token: "string",
                    regex: '.+'
                }
            ]
        };
    };

    oop.inherits(FocusHighlightRules, TextHighlightRules);

    exports.FocusHighlightRules = FocusHighlightRules;
});

ace.define("ace/mode/matching_brace_outdent", ["require", "exports", "module", "ace/range"], function (acequire, exports, module) {
    "use strict";

    var Range = acequire("../range").Range;

    var MatchingBraceOutdent = function () { };

    (function () {

        this.checkOutdent = function (line, input) {
            if (! /^\s+$/.test(line))
                return false;

            return /^\s*\}/.test(input);
        };

        this.autoOutdent = function (doc, row) {
            var line = doc.getLine(row);
            var match = line.match(/^(\s*\})/);

            if (!match) return 0;

            var column = match[1].length;
            var openBracePos = doc.findMatchingBracket({ row: row, column: column });

            if (!openBracePos || openBracePos.row == row) return 0;

            var indent = this.$getIndent(doc.getLine(openBracePos.row));
            doc.replace(new Range(row, 0, row, column - 1), indent);
        };

        this.$getIndent = function (line) {
            return line.match(/^\s*/)[0];
        };

    }).call(MatchingBraceOutdent.prototype);

    exports.MatchingBraceOutdent = MatchingBraceOutdent;
});

ace.define("ace/mode/focus", ["require", "exports", "module", "ace/lib/oop", "ace/mode/text", "ace/mode/focus_highlight_rules", "ace/mode/matching_brace_outdent", "ace/range"], function (acequire, exports, module) {
    "use strict";

    var oop = acequire("../lib/oop");
    var TextMode = acequire("./text").Mode;
    var FocusHighlightRules = acequire("./focus_highlight_rules").FocusHighlightRules;
    var MatchingBraceOutdent = acequire("./matching_brace_outdent").MatchingBraceOutdent;
    var Range = acequire("../range").Range;

    var Mode = function () {
        this.HighlightRules = FocusHighlightRules;
        this.$behaviour = this.$defaultBehaviour;

        this.$outdent = new MatchingBraceOutdent();
    };
    oop.inherits(Mode, TextMode);

    var indenter = /(?:[({[=:]|[-=]>|\b(?:else|try|with))\s*$/;

    (function () {

        this.toggleCommentLines = function (state, doc, startRow, endRow) {
            var i, line;
            var outdent = true;
            var re = /^\s*\(\*(.*)\*\)/;

            for (i = startRow; i <= endRow; i++) {
                if (!re.test(doc.getLine(i))) {
                    outdent = false;
                    break;
                }
            }

            var range = new Range(0, 0, 0, 0);
            for (i = startRow; i <= endRow; i++) {
                line = doc.getLine(i);
                range.start.row = i;
                range.end.row = i;
                range.end.column = line.length;

                doc.replace(range, outdent ? line.match(re)[1] : "(*" + line + "*)");
            }
        };

        this.getNextLineIndent = function (state, line, tab) {
            var indent = this.$getIndent(line);
            var tokens = this.getTokenizer().getLineTokens(line, state).tokens;

            if (!(tokens.length && tokens[tokens.length - 1].type === 'comment') &&
                state === 'start' && indenter.test(line))
                indent += tab;
            return indent;
        };

        this.checkOutdent = function (state, line, input) {
            return this.$outdent.checkOutdent(line, input);
        };

        this.autoOutdent = function (state, doc, row) {
            this.$outdent.autoOutdent(doc, row);
        };

        this.$id = "ace/mode/focus";
    }).call(Mode.prototype);

    exports.Mode = Mode;
});
