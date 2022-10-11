from common import *


def parse_data(filename):
    """
    Map from number of bits to (list of X values, list of Y values)
    """
    data = {}
    with open(filename) as f:
        lines = f.read().split('\n')
    bits = None
    xs = None
    ys = None

    def add_bits():
        if bits is not None:
            data[bits] = (xs, ys)

    for line in lines:
        if line == '' or 'time' in line:
            continue
        elif 'bits' in line:
            add_bits()
            bits = int(line.split()[0])
            xs = []
            ys = []
        else:
            split = line.split()
            xs.append(int(split[0]))
            ys.append(int(split[1]))
    add_bits()
    return data


def plot_graph(filename, pdf, xlabel, keys=None):
    data = parse_data(filename)
    keys = keys if keys is not None else sorted([k for k in data.keys()])
    plt.clf()
    for (i, key) in enumerate(keys):
        label = '{} bits'.format(key)
        (x, y) = data[key]
        plt.plot(x, y, label=label, marker=MARKERS[i])
    plt.xlabel(xlabel)
    if pdf is not None:
        save_pdf(pdf)


def plot_legend(filename, pdf, keys=None):
    sns.set_context("paper", font_scale=2,  rc=paper_rc)
    plot_graph(filename=filename, pdf=None, xlabel='', keys=keys)
    plt.legend(loc='upper center', bbox_to_anchor=(0.5, 1.25), ncol=4)
    bbox = Bbox.from_bounds(-0.5, 4.55, 7.55, 0.55)
    save_pdf(pdf, bbox_inches=bbox)

keys = [16,24,32]
plot_graph(filename='insertion.txt', pdf='figure5.pdf',
           xlabel='Threshold (n=1000)', keys=keys)
plot_graph(filename='decode.txt', pdf='figure6.pdf',
           xlabel='Missing Packets (n=1000,t=20)', keys=keys)
plot_legend(filename='insertion.txt', pdf='legend.pdf', keys=keys)
