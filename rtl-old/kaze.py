import ctypes
import inspect

# Lib dependencies

class CodeWriter:
    def __init__(self):
        self.buffer = ''
        self.indent_level = 0

    def indent(self):
        self.indent_level += 1

    def unindent(self):
        self.indent_level -= 1

    def append_indent(self):
        for _ in range(self.indent_level):
            self.buffer += '    '

    def append_newline(self):
        self.buffer += '\n'

    def append(self, s):
        self.buffer += s

    def append_line(self, s):
        self.append_indent()
        self.append(s)
        self.append_newline()

class CodegenContext:
    def __init__(self):
        self.node_names = dict()
        self.nodes_decl_generated = set()

        self.queued_assignments = []

    def get_node_name(self, node):
        if not node in self.node_names:
            self.node_names[node] = 'node{}'.format(len(self.node_names))
        return self.node_names[node]

    def node_has_decl_generated(self, node):
        return node in self.nodes_decl_generated

    def mark_node_has_decl_generated(self, node):
        self.nodes_decl_generated.add(node)

    def queue_assignment(self, name, value):
        self.queued_assignments.append((name, value))

# Lib classes

class Signal:
    def num_bits(self):
        raise Exception('num_bits not implemented')

    def ensure_num_bits(self, expected):
        if self.num_bits() != expected:
            raise Exception('signal has wrong number of bits (expected {}, have {})'.format(expected, self.num_bits()))

class Source(Signal):
    def gen_node_decls(self, c, w):
        raise Exception('gen_node_decls not implemented')

    def gen_assign_expr(self, c, w):
        raise Exception('gen_assign_expr not implemented')

    def __invert__(self):
        return UnOp(self, '~')

    def eq(self, other):
        return BinOp(self, other, '==', 1)

    def ne(self, other):
        return BinOp(self, other, '!=', 1)

    def __and__(self, other):
        return BinOp(self, other, '&')

    def __or__(self, other):
        return BinOp(self, other, '|')

    def __xor__(self, other):
        return BinOp(self, other, '^')

    def __add__(self, other):
        return BinOp(self, other, '+', max(self.num_bits(), other.num_bits()) + 1)

    def __lt__(self, other):
        return BinOp(self, other, '<', 1)

    def __le__(self, other):
        return BinOp(self, other, '<=', 1)

    def __gt__(self, other):
        return BinOp(self, other, '>', 1)

    def __ge__(self, other):
        return BinOp(self, other, '>=', 1)

    def lt_signed(self, other):
        return BinOp(self, other, '<', 1, True)

    def le_signed(self, other):
        return BinOp(self, other, '<=', 1, True)

    def gt_signed(self, other):
        return BinOp(self, other, '>', 1, True)

    def ge_signed(self, other):
        return BinOp(self, other, '>=', 1, True)

    def bit(self, index):
        return Bit(self, index)

    def bits(self, range_high, range_low):
        return Bits(self, range_high, range_low)

    def repeat(self, count):
        return Repeat(self, count)

    def concat(self, other):
        return Concat(self, other)

class Sink(Signal):
    def is_driven(self):
        raise Exception('is_driven not implemented')

def ensure_source(source):
    if not isinstance(source, Source):
        raise Exception('source must be a Source')

def ensure_sink(sink):
    if not isinstance(sink, Sink):
        raise Exception('sink must be a Sink')

def ensure_compatibility(source, sink):
    ensure_source(source)
    ensure_sink(sink)
    if source.num_bits() != sink.num_bits():
        raise Exception('source and sink have different numbers of bits ({} and {}, respectively)'.format(source.num_bits(), sink.num_bits()))

class Register(Source, Sink):
    def __init__(self, num_bits, initial_value):
        if num_bits <= 0:
            raise Exception('num_bits must be a nonzero integer')
        # TODO: Bounds check for initial_value

        self.next = None
        self._num_bits = num_bits
        self.initial_value = initial_value

    def num_bits(self):
        return self._num_bits

    def gen_node_decls(self, c, w):
        if not self.is_driven():
            raise Exception('register next is not driven')
        if c.node_has_decl_generated(self):
            return
        c.mark_node_has_decl_generated(self)
        self.next.gen_node_decls(c, w)
        node_name = c.get_node_name(self)
        w.append_indent()
        w.append('logic ')
        if self.num_bits() > 1:
            w.append('[{}:{}] '.format(self.num_bits() - 1, 0))
        w.append('{};'.format(node_name))
        w.append_newline()
        w.append_indent()
        w.append('logic ')
        if self.num_bits() > 1:
            w.append('[{}:{}] '.format(self.num_bits() - 1, 0))
        w.append('{}_next;'.format(node_name))
        w.append_newline()
        c.queue_assignment('{}_next'.format(node_name), self.next)
        w.append_line('always_ff @(posedge clk) begin')
        w.indent()
        w.append_line('if (!reset_n) begin')
        w.indent()
        w.append_line('{} <= {}\'h{:x};'.format(node_name, self.num_bits(), self.initial_value if self.initial_value is not None else 0))
        w.unindent()
        w.append_line('end')
        w.append_line('else begin')
        w.indent()
        w.append_line('{} <= {}_next;'.format(node_name, node_name))
        w.unindent()
        w.append_line('end')
        w.unindent()
        w.append_line('end')
        w.append_newline()

    def gen_assign_expr(self, c, w):
        w.append(c.get_node_name(self))

    def is_driven(self):
        return self.next is not None

    def drive_next_with(self, next):
        if self.is_driven():
            raise Exception('register next is already driven')
        ensure_compatibility(next, self)

        self.next = next;

# Included batteries

def reg(num_bits, initial_value = None):
    return Register(num_bits, initial_value)

class Low(Source):
    def num_bits(self):
        return 1

    def gen_node_decls(self, c, w):
        pass

    def gen_assign_expr(self, c, w):
        w.append('1\'h0')

LOW = Low()

class High(Source):
    def num_bits(self):
        return 1

    def gen_node_decls(self, c, w):
        pass

    def gen_assign_expr(self, c, w):
        w.append('1\'h1')

HIGH = High()

class UnOp(Source):
    def __init__(self, source, op):
        self.source = source
        self.op = op

    def num_bits(self):
        return self.source.num_bits()

    def gen_node_decls(self, c, w):
        self.source.gen_node_decls(c, w)

    def gen_assign_expr(self, c, w):
        w.append(self.op)
        self.source.gen_assign_expr(c, w)

class BinOp(Source):
    def __init__(self, a, b, op, num_bits = None, signed = False):
        if a.num_bits() != b.num_bits():
            raise Exception('sources have different numbers of bits ({} and {}, respectively)'.format(a.num_bits(), b.num_bits()))

        self.a = a
        self.b = b
        self.op = op

        if num_bits is None:
            num_bits = a.num_bits()
        self._num_bits = num_bits

        self.signed = signed

    def num_bits(self):
        return self._num_bits

    def gen_node_decls(self, c, w):
        self.a.gen_node_decls(c, w)
        self.b.gen_node_decls(c, w)

    def gen_assign_expr(self, c, w):
        w.append('('.format(self.op))
        if self.signed:
            w.append('$signed(')
        self.a.gen_assign_expr(c, w)
        if self.signed:
            w.append(')')
        w.append(' {} '.format(self.op))
        if self.signed:
            w.append('$signed(')
        self.b.gen_assign_expr(c, w)
        if self.signed:
            w.append(')')
        w.append(')')

class Bit(Source):
    def __init__(self, source, index):
        if index < 0 or index >= source.num_bits():
            raise Exception('index out of range for source (expected to be in [0, {}))'.format(source.num_bits()))

        self.source = source
        self.index = index

    def num_bits(self):
        return 1

    def gen_node_decls(self, c, w):
        if c.node_has_decl_generated(self):
            return
        c.mark_node_has_decl_generated(self)
        self.source.gen_node_decls(c, w)
        node_name = c.get_node_name(self)
        w.append_indent()
        w.append('logic ')
        if self.source.num_bits() > 1:
            w.append('[{}:{}] '.format(self.source.num_bits() - 1, 0))
        w.append('{};'.format(node_name))
        w.append_newline()
        c.queue_assignment(node_name, self.source)

    def gen_assign_expr(self, c, w):
        w.append('{}[{}]'.format(c.get_node_name(self), self.index))

class Bits(Source):
    def __init__(self, source, range_high, range_low):
        if range_high < range_low:
            raise Exception('range_high cannot be lower than range_low')
        if range_high < 0 or range_high >= source.num_bits():
            raise Exception('range_high out of range for source (expected to be in [0, {}))'.format(source.num_bits()))
        if range_low < 0 or range_low >= source.num_bits():
            raise Exception('range_low out of range for source (expected to be in [0, {}))'.format(source.num_bits()))

        self.source = source
        self.range_high = range_high
        self.range_low = range_low

    def num_bits(self):
        return self.range_high - self.range_low + 1

    def gen_node_decls(self, c, w):
        if c.node_has_decl_generated(self):
            return
        c.mark_node_has_decl_generated(self)
        self.source.gen_node_decls(c, w)
        node_name = c.get_node_name(self)
        w.append_indent()
        w.append('logic ')
        if self.source.num_bits() > 1:
            w.append('[{}:{}] '.format(self.source.num_bits() - 1, 0))
        w.append('{};'.format(node_name))
        w.append_newline()
        c.queue_assignment(node_name, self.source)

    def gen_assign_expr(self, c, w):
        w.append('{}[{}:{}]'.format(c.get_node_name(self), self.range_high, self.range_low))

class Ternary(Source):
    def __init__(self, a, b, sel):
        ensure_source(a)
        ensure_source(b)
        if a.num_bits() != b.num_bits():
            raise Exception('sources have different numbers of bits ({} and {}, respectively)'.format(a.num_bits(), b.num_bits()))
        ensure_source(sel)
        sel.ensure_num_bits(1)

        self.a = a
        self.b = b
        self.sel = sel

    def num_bits(self):
        return self.a.num_bits()

    def gen_node_decls(self, c, w):
        self.b.gen_node_decls(c, w)
        self.a.gen_node_decls(c, w)
        self.sel.gen_node_decls(c, w)

    def gen_assign_expr(self, c, w):
        w.append('(')
        self.sel.gen_assign_expr(c, w)
        w.append(' ? ')
        self.b.gen_assign_expr(c, w)
        w.append(' : ')
        self.a.gen_assign_expr(c, w)
        w.append(')')

class Repeat(Source):
    def __init__(self, source, count):
        ensure_source(source)
        if count <= 0:
            raise Exception('count must be greater than 0')
        self.source = source
        self.count = count

    def num_bits(self):
        return self.source.num_bits() * self.count

    def gen_node_decls(self, c, w):
        self.source.gen_node_decls(c, w)

    def gen_assign_expr(self, c, w):
        w.append('{{{}{{'.format(self.count))
        self.source.gen_assign_expr(c, w)
        w.append('}}')

def repeat(source, count):
    return Repeat(source, count)

class Concat(Source):
    def __init__(self, a, b):
        ensure_source(a)
        ensure_source(b)
        self.a = a
        self.b = b

    def num_bits(self):
        return self.a.num_bits() + self.b.num_bits()

    def gen_node_decls(self, c, w):
        self.a.gen_node_decls(c, w)
        self.b.gen_node_decls(c, w)

    def gen_assign_expr(self, c, w):
        w.append('{')
        self.a.gen_assign_expr(c, w)
        w.append(', ')
        self.b.gen_assign_expr(c, w)
        w.append('}')

def concat(a, b):
    return Concat(a, b)

class Literal(Source):
    def __init__(self, value, num_bits):
        # TODO: Bounds-check for value (this might need to be signed!)
        if num_bits <= 0:
            raise Exception('num_bits must be greater than 0')
        self.value = value
        self._num_bits = num_bits

    def num_bits(self):
        return self._num_bits

    def gen_node_decls(self, c, w):
        pass

    def gen_assign_expr(self, c, w):
        w.append('{}\'h{:x}'.format(self.num_bits(), self.value))

def lit(value, num_bits):
    return Literal(value, num_bits)

def mux(a, b, sel):
    return Ternary(a, b, sel)

class If:
    def __init__(self, sel):
        self.sel = sel

    def __enter__(self):
        self.locals = dict(inspect.currentframe().f_back.f_locals)
        # TODO: Error if locals doesn't contain any Sources

    def __exit__(self, exc_type, exc_val, exc_tb):
        frame = inspect.currentframe().f_back
        new_locals = dict(frame.f_locals)
        for name, value in new_locals.items():
            if name not in self.locals:
                raise Exception('New local added in If block: {}. New locals are not allowed in If blocks.'.format(name))
            old_value = self.locals[name]
            if value != old_value:
                frame.f_locals[name] = mux(old_value, value, self.sel)
            ctypes.pythonapi.PyFrame_LocalsToFast(ctypes.py_object(frame), ctypes.c_int(0))

class Module:
    def __init__(self, name):
        self.name = name

        self.inputs = dict()
        self.outputs = dict()

    def input(self, name, num_bits):
        # TODO: Error if name already exists in this context
        ret = Input(name, num_bits)
        self.inputs[name] = ret
        return ret

    def output(self, name, source):
        # TODO: Error if name already exists in this context
        ret = Output(name, source)
        self.outputs[name] = ret

    def gen_code(self, c, w):
        w.append_line('module {}('.format(self.name))
        w.indent()

        w.append_line('input wire logic reset_n,')
        w.append_indent()
        w.append('input wire logic clk')
        if len(self.inputs) > 0 or len(self.outputs) > 0:
            w.append(',')
            w.append_newline()
        w.append_newline()
        for i, input in enumerate(self.inputs.values()):
            w.append_indent()
            w.append('input wire logic ')
            if input.num_bits() > 1:
                w.append('[{}:{}] '.format(input.num_bits() - 1, 0))
            w.append(input.name)
            if len(self.outputs) > 0 or i < len(self.inputs) - 1:
                w.append(',')
            w.append_newline()
        for i, output in enumerate(self.outputs.values()):
            w.append_indent()
            w.append('output wire logic ')
            if output.num_bits() > 1:
                w.append('[{}:{}] '.format(output.num_bits() - 1, 0))
            w.append(output.name)
            if i < len(self.outputs) - 1:
                w.append(',')
            w.append_newline()
        w.append_line(');')
        w.append_newline()

        for output in self.outputs.values():
            output.gen_node_decls(c, w)
            c.queue_assignment(output.name, output)

        for name, value in c.queued_assignments:
            w.append_indent()
            w.append('assign {} = '.format(name))
            value.gen_assign_expr(c, w)
            w.append(';')
            w.append_newline()

        w.append_newline()
        w.unindent()
        w.append_line('endmodule')
        w.append_newline()

class Input(Source):
    def __init__(self, name, num_bits):
        self.name = name
        self._num_bits = num_bits

    def num_bits(self):
        return self._num_bits

    def gen_node_decls(self, c, w):
        pass

    def gen_assign_expr(self, c, w):
        w.append(self.name)

class Output(Sink):
    def __init__(self, name, source):
        self.name = name
        self.source = source

        ensure_compatibility(source, self)

    def num_bits(self):
        return self.source.num_bits()

    def is_driven(self):
        return True

    def gen_node_decls(self, c, w):
        self.source.gen_node_decls(c, w)

    def gen_assign_expr(self, c, w):
        self.source.gen_assign_expr(c, w)
